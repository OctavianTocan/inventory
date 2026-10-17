//! `mm_core::host::Bucket` over an R2 bucket (`worker/bucket-r2.mjs`).

use async_trait::async_trait;
use mm_core::host::{Bucket, ByteSpan, ListedObject, ObjectInfo, StoredObject};
use mm_core::http::{Body, Headers};
use mm_core::Result;
use mm_js::json;
use wasm_bindgen::JsCast;

use crate::util::{call, js_error, worker_error};

/// The Worker's R2 binding.
pub struct R2Bucket(pub worker::Bucket);

fn stored_headers(object: &worker::Object) -> Result<Option<Headers>> {
    let metadata = object.custom_metadata().map_err(worker_error)?;
    let Some(text) = metadata.get("headers") else { return Ok(None) };
    let value = json::parse(text)?;
    let Some(map) = value.as_object() else { return Ok(None) };
    Ok(Some(Headers::from_pairs(map.iter().map(|(name, value)| (name.into_owned(), value.to_js_string())))))
}

fn encoded_headers(headers: &Headers) -> Result<String> {
    let map = headers.iter().map(|(name, value)| (name.to_owned(), mm_js::Value::from(value))).collect::<mm_js::Object>();
    Ok(json::stringify(&mm_js::Value::Object(map))?.unwrap_or_default())
}

/// `stream.pipeThrough(new FixedLengthStream(length))`, which R2 needs to take a stream of a known size.
fn fixed_length(stream: &web_sys::ReadableStream, length: u64) -> Result<web_sys::ReadableStream> {
    let sized = match u32::try_from(length) {
        Ok(small) => worker::worker_sys::FixedLengthStream::new(small),
        Err(_) => worker::worker_sys::FixedLengthStream::new_big_int(js_sys::BigInt::from(length)),
    }
    .map_err(js_error)?;
    Ok(call(stream, "pipeThrough", &[sized.into()])?.unchecked_into())
}

#[async_trait(?Send)]
impl Bucket for R2Bucket {
    async fn head(&self, key: &str) -> Result<Option<ObjectInfo>> {
        let Some(object) = self.0.head(key).await.map_err(worker_error)? else { return Ok(None) };
        Ok(Some(ObjectInfo { size: object.size(), headers: stored_headers(&object)? }))
    }

    async fn read(&self, key: &str, range: Option<ByteSpan>) -> Result<Option<StoredObject>> {
        if range.is_some_and(|span| span.length == 0) {
            let Some(object) = self.0.head(key).await.map_err(worker_error)? else { return Ok(None) };
            return Ok(Some(StoredObject { size: object.size(), headers: stored_headers(&object)?, body: Body::Empty.into_stream() }));
        }
        let mut get = self.0.get(key);
        if let Some(span) = range {
            get = get.range(worker::Range::OffsetWithLength { offset: span.offset, length: span.length });
        }
        let Some(object) = get.execute().await.map_err(worker_error)? else { return Ok(None) };
        let body = match object.body().map(|body| body.response_body()).transpose().map_err(worker_error)? {
            Some(worker::ResponseBody::Stream(raw)) => crate::http::byte_stream(raw),
            _ => Body::Empty.into_stream(),
        };
        Ok(Some(StoredObject { size: object.size(), headers: stored_headers(&object)?, body }))
    }

    async fn bytes(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let Some(object) = self.0.get(key).execute().await.map_err(worker_error)? else { return Ok(None) };
        match object.body() {
            Some(body) => Ok(Some(body.bytes().await.map_err(worker_error)?)),
            None => Ok(Some(Vec::new())),
        }
    }

    async fn write(&self, key: &str, body: Body, length: Option<u64>, headers: Option<&Headers>) -> Result<()> {
        let data: worker::Data = match (body, length) {
            (Body::Empty, _) => worker::Data::Bytes(Vec::new()),
            (Body::Bytes(bytes), _) => worker::Data::Bytes(bytes),
            (Body::Stream(stream), Some(length)) => match crate::http::unread(&stream) {
                Some(raw) => worker::Data::ReadableStream(fixed_length(&raw, length)?),
                None => {
                    let chunks = futures_util::StreamExt::map(stream, |chunk| chunk.map_err(|error| worker::Error::RustError(error.to_string())));
                    worker::Data::Stream(worker::FixedLengthStream::wrap(chunks, length))
                }
            },
            (stream @ Body::Stream(_), None) => worker::Data::Bytes(stream.bytes().await?),
        };
        let mut put = self.0.put(key, data);
        if let Some(headers) = headers {
            put = put.custom_metadata([("headers".to_owned(), encoded_headers(headers)?)].into_iter().collect::<std::collections::HashMap<_, _>>());
        }
        put.execute().await.map_err(worker_error)?;
        Ok(())
    }

    async fn remove(&self, key: &str) -> Result<()> {
        self.0.delete(key).await.map_err(worker_error)
    }

    async fn list(&self, prefix: &str) -> Result<Vec<ListedObject>> {
        let mut listed = Vec::new();
        let mut cursor: Option<String> = None;
        loop {
            let mut list = self.0.list().prefix(prefix);
            if let Some(cursor) = cursor.take() {
                list = list.cursor(cursor);
            }
            let page = list.execute().await.map_err(worker_error)?;
            listed.extend(page.objects().iter().map(|object| ListedObject { key: object.key(), size: object.size() }));
            match page.truncated().then(|| page.cursor()).flatten() {
                Some(next) => cursor = Some(next),
                None => break,
            }
        }
        listed.sort_by(|a, b| a.key.cmp(&b.key));
        Ok(listed)
    }
}
