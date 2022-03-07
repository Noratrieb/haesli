use axum::{
    body::Body,
    http::{header, Request, Response, StatusCode},
};
use mime_guess::mime;
use std::{
    collections::HashMap,
    fmt::{Debug, Formatter},
    future,
    io::Cursor,
    path::Path,
    task::{Context, Poll},
};
use tracing::trace;
use zip::ZipArchive;

#[derive(Debug, Clone)]
enum StaticFileKind {
    File { mime: mime::Mime },
    Directory,
}

#[derive(Clone)]
struct StaticFile {
    data: Vec<u8>,
    kind: StaticFileKind,
}

impl Debug for StaticFile {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StaticFile")
            .field("kind", &self.kind)
            .field("data", &format!("[{} bytes]", self.data.len()))
            .finish()
    }
}

#[derive(Debug, Clone)]
pub struct StaticFileService {
    files: HashMap<String, StaticFile>,
}

impl StaticFileService {
    /// Creates a new static file service from zip data. This is a blocking operation!
    #[tracing::instrument(skip(zip))]
    pub fn new(zip: &[u8]) -> anyhow::Result<Self> {
        let mut archive = ZipArchive::new(Cursor::new(zip))?;

        let mut files = HashMap::with_capacity(archive.len());

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let mut data = Vec::with_capacity(usize::try_from(file.size())?);
            std::io::copy(&mut file, &mut data)?;

            trace!(name = %file.name(), size = %file.size(),"Unpacking dashboard frontend file");

            let path = Path::new(file.name());

            let kind = if file.is_dir() {
                StaticFileKind::Directory
            } else {
                let mime = match path.extension() {
                    Some(ext) => {
                        mime_guess::from_ext(&ext.to_string_lossy()).first_or_octet_stream()
                    }
                    None => mime::APPLICATION_OCTET_STREAM,
                };
                StaticFileKind::File { mime }
            };

            files.insert(
                file.name().trim_start_matches('/').to_owned(),
                StaticFile { data, kind },
            );
        }

        trace!(?files, "Created StaticFileService");

        Ok(Self { files })
    }

    fn call_inner(&mut self, req: Request<Body>) -> Result<Response<Body>, anyhow::Error> {
        trace!(?req, "Got request for static file");

        let path = req.uri().path().trim_start_matches('/');

        let entry = self.files.get(path);

        match entry {
            Some(file) => {
                if let StaticFileKind::File { mime } = &file.kind {
                    trace!(%path, "Found file");
                    Ok(Response::builder()
                        .status(StatusCode::OK)
                        .header(header::CONTENT_TYPE, mime.essence_str())
                        .body(Body::from(file.data.clone()))?)
                } else {
                    trace!(%path, "Found directory, trying to append index.html");

                    let new_path = if path.is_empty() {
                        "index.html".to_owned()
                    } else {
                        let new_path = path.trim_end_matches('/');
                        format!("{}/index.html", new_path)
                    };

                    match self.files.get(&new_path) {
                        Some(file) => {
                            trace!(%new_path, "Found index.html");
                            if let StaticFileKind::File { mime } = &file.kind {
                                Ok(Response::builder()
                                    .status(StatusCode::OK)
                                    .header(header::CONTENT_TYPE, mime.essence_str())
                                    .body(Body::from(file.data.clone()))?)
                            } else {
                                unreachable!()
                            }
                        }
                        None => {
                            trace!(%new_path, "Did not find index.html");
                            Ok(Response::builder()
                                .status(StatusCode::NOT_FOUND)
                                .body(Body::empty())?)
                        }
                    }
                }
            }
            None => {
                trace!(%path, "Did not find file");
                Ok(Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(Body::empty())?)
            }
        }
    }
}

impl tower::Service<Request<Body>> for StaticFileService {
    type Response = Response<Body>;
    type Error = anyhow::Error;
    type Future = future::Ready<Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        future::ready(self.call_inner(req))
    }
}
