use quick_xml::Reader;
use quick_xml::events::Event as XmlEvent;
use reqwest::{Method, StatusCode};

use crate::error::{ArgvusError, Result};

use super::{CalendarCollection, RemoteObject};

pub struct CalDavClient {
  base_url: String,
  username: String,
  password: String,
  http: reqwest::Client,
}

impl CalDavClient {
  pub fn new(
    base_url: impl Into<String>,
    username: impl Into<String>,
    password: impl Into<String>,
  ) -> Result<Self> {
    let http = reqwest::Client::builder()
      .user_agent("argvus-taskbar-calendar/0.1.0")
      .build()?;
    Ok(Self {
      base_url: base_url.into().trim_end_matches('/').to_string(),
      username: username.into(),
      password: password.into(),
      http,
    })
  }

  pub async fn discover_collections(&self) -> Result<Vec<CalendarCollection>> {
    let body = r#"<?xml version="1.0" encoding="utf-8"?>
<d:propfind xmlns:d="DAV:" xmlns:cs="http://calendarserver.org/ns/">
  <d:prop>
    <d:displayname/>
    <cs:getctag/>
  </d:prop>
</d:propfind>"#;
    let response = self
      .dav("PROPFIND", &self.base_url, Some(body), Some("1"))
      .await?;
    if !response.status().is_success() && response.status() != StatusCode::MULTI_STATUS {
      return Err(ArgvusError::CalDav(format!(
        "collection discovery failed with {}",
        response.status()
      )));
    }
    parse_collections(&response.text().await?)
  }

  pub async fn list_objects(&self, collection_url: &str) -> Result<Vec<RemoteObject>> {
    let body = r#"<?xml version="1.0" encoding="utf-8"?>
<c:calendar-query xmlns:d="DAV:" xmlns:c="urn:ietf:params:xml:ns:caldav">
  <d:prop>
    <d:getetag/>
    <c:calendar-data/>
  </d:prop>
  <c:filter>
    <c:comp-filter name="VCALENDAR">
      <c:comp-filter name="VEVENT"/>
    </c:comp-filter>
  </c:filter>
</c:calendar-query>"#;
    let response = self
      .dav("REPORT", collection_url, Some(body), Some("1"))
      .await?;
    if !response.status().is_success() && response.status() != StatusCode::MULTI_STATUS {
      return Err(ArgvusError::CalDav(format!(
        "calendar query failed with {}",
        response.status()
      )));
    }
    parse_remote_objects(&response.text().await?)
  }

  pub async fn put_object(
    &self,
    href: &str,
    ics: String,
    etag: Option<&str>,
  ) -> Result<Option<String>> {
    let mut request = self
      .http
      .request(Method::PUT, href)
      .basic_auth(&self.username, Some(&self.password))
      .header("Content-Type", "text/calendar; charset=utf-8")
      .body(ics);
    if let Some(etag) = etag {
      request = request.header("If-Match", etag);
    } else {
      request = request.header("If-None-Match", "*");
    }
    let response = request.send().await?;
    if response.status() == StatusCode::PRECONDITION_FAILED {
      return Err(ArgvusError::CalDav(
        "remote event changed; preserving local conflict copy".to_string(),
      ));
    }
    if !response.status().is_success() {
      return Err(ArgvusError::CalDav(format!(
        "PUT failed with {}",
        response.status()
      )));
    }
    Ok(
      response
        .headers()
        .get(reqwest::header::ETAG)
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned),
    )
  }

  pub async fn delete_object(&self, href: &str, etag: Option<&str>) -> Result<()> {
    let mut request = self
      .http
      .request(Method::DELETE, href)
      .basic_auth(&self.username, Some(&self.password));
    if let Some(etag) = etag {
      request = request.header("If-Match", etag);
    }
    let response = request.send().await?;
    if !response.status().is_success() && response.status() != StatusCode::NOT_FOUND {
      return Err(ArgvusError::CalDav(format!(
        "DELETE failed with {}",
        response.status()
      )));
    }
    Ok(())
  }

  async fn dav(
    &self,
    method: &str,
    url: &str,
    body: Option<&str>,
    depth: Option<&str>,
  ) -> Result<reqwest::Response> {
    let method = Method::from_bytes(method.as_bytes())
      .map_err(|err| ArgvusError::CalDav(format!("invalid DAV method: {err}")))?;
    let mut request = self
      .http
      .request(method, url)
      .basic_auth(&self.username, Some(&self.password))
      .header("Content-Type", "application/xml; charset=utf-8");
    if let Some(depth) = depth {
      request = request.header("Depth", depth);
    }
    if let Some(body) = body {
      request = request.body(body.to_string());
    }
    Ok(request.send().await?)
  }
}

fn parse_collections(xml: &str) -> Result<Vec<CalendarCollection>> {
  let mut reader = Reader::from_str(xml);
  reader.config_mut().trim_text(true);
  let mut href = None;
  let mut display_name = None;
  let mut ctag = None;
  let mut current = String::new();
  let mut collections = Vec::new();
  loop {
    match reader.read_event() {
      Ok(XmlEvent::Start(element)) => current = local_name(element.name().as_ref()).to_string(),
      Ok(XmlEvent::Text(text)) => match current.as_str() {
        "href" => href = Some(text.decode().unwrap_or_default().to_string()),
        "displayname" => display_name = Some(text.decode().unwrap_or_default().to_string()),
        "getctag" => ctag = Some(text.decode().unwrap_or_default().to_string()),
        _ => {}
      },
      Ok(XmlEvent::End(element)) if local_name(element.name().as_ref()) == "response" => {
        if let Some(href) = href.take() {
          collections.push(CalendarCollection {
            display_name: display_name.take().unwrap_or_else(|| href.clone()),
            href,
            ctag: ctag.take(),
          });
        }
      }
      Ok(XmlEvent::Eof) => break,
      Err(err) => return Err(ArgvusError::CalDav(format!("invalid DAV XML: {err}"))),
      _ => {}
    }
  }
  Ok(collections)
}

fn parse_remote_objects(xml: &str) -> Result<Vec<RemoteObject>> {
  let mut reader = Reader::from_str(xml);
  reader.config_mut().trim_text(false);
  let mut href = None;
  let mut etag = None;
  let mut ics = None;
  let mut current = String::new();
  let mut objects = Vec::new();
  loop {
    match reader.read_event() {
      Ok(XmlEvent::Start(element)) => current = local_name(element.name().as_ref()).to_string(),
      Ok(XmlEvent::Text(text)) => match current.as_str() {
        "href" => href = Some(text.decode().unwrap_or_default().to_string()),
        "getetag" => etag = Some(text.decode().unwrap_or_default().to_string()),
        "calendar-data" => ics = Some(text.decode().unwrap_or_default().to_string()),
        _ => {}
      },
      Ok(XmlEvent::End(element)) if local_name(element.name().as_ref()) == "response" => {
        if let (Some(href), Some(ics)) = (href.take(), ics.take()) {
          objects.push(RemoteObject {
            href,
            etag: etag.take(),
            ics,
            updated_at: chrono::Utc::now(),
          });
        }
      }
      Ok(XmlEvent::Eof) => break,
      Err(err) => return Err(ArgvusError::CalDav(format!("invalid DAV XML: {err}"))),
      _ => {}
    }
  }
  Ok(objects)
}

fn local_name(name: &[u8]) -> &str {
  let name = std::str::from_utf8(name).unwrap_or_default();
  name
    .rsplit_once(':')
    .map(|(_, local)| local)
    .unwrap_or(name)
}
