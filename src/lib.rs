use std::io;
use std::sync::Arc;

use async_graphql::InputObject;
use async_graphql::Object;

use rs_wikipages2struct::Page as PageEx;

pub use async_graphql;
pub use async_trait;
pub use rs_wikipages2struct;

#[derive(InputObject)]
pub struct BasicFilter {
    pub namespace: Option<String>,
    pub title: Option<String>,
    pub has_redirect: Option<bool>,
}

impl BasicFilter {
    pub fn filter_ex_namespace(&self, p: &PageEx) -> bool {
        let ns: Option<&str> = p.namespace.as_deref();
        let np = (&self.namespace, ns);
        match np {
            (None, _) => true,
            (Some(_), None) => false,
            (Some(f), Some(v)) => f == v,
        }
    }

    pub fn filter_ex_title(&self, p: &PageEx) -> bool {
        let title: Option<&str> = p.title.as_deref();
        let nt = (&self.title, title);
        match nt {
            (None, _) => true,
            (Some(_), None) => false,
            (Some(f), Some(v)) => f == v,
        }
    }

    pub fn filter_redirect(&self, p: &PageEx) -> bool {
        match self.has_redirect {
            None => true,
            Some(expected) => p.redirect.is_some() == expected,
        }
    }

    pub fn filter_ex(&self, p: &PageEx) -> bool {
        self.filter_ex_namespace(p) && self.filter_ex_title(p) && self.filter_redirect(p)
    }
}

#[async_trait::async_trait]
pub trait PagesStringSource: Sync + Send + 'static {
    async fn offset2pages_string(&self, offset: u64, size: u64) -> Result<String, io::Error>;
}

#[async_trait::async_trait]
pub trait PagesStringToPages: Sync + Send + 'static {
    async fn pages_string2pages(&self, pages: &str) -> Result<Vec<PageEx>, io::Error>;

    async fn pages_string2pages_filtered(
        &self,
        pages: &str,
        f: &BasicFilter,
    ) -> Result<Vec<PageEx>, io::Error> {
        let all = self.pages_string2pages(pages).await?;
        let filtered: Vec<PageEx> = all.into_iter().filter(|p| f.filter_ex(p)).collect();
        Ok(filtered)
    }
}

#[async_trait::async_trait]
pub trait PagesExSource: Sync + Send + 'static {
    async fn offset2pages(&self, offset: u64, size: u64) -> Result<Vec<PageEx>, io::Error>;
    async fn offset2pages_filtered(
        &self,
        offset: u64,
        size: u64,
        f: &BasicFilter,
    ) -> Result<Vec<PageEx>, io::Error>;
}

pub struct PagesExSrc {
    pub source: Arc<Box<dyn PagesStringSource>>,
    pub s2page: Arc<Box<dyn PagesStringToPages>>,
}

#[async_trait::async_trait]
impl PagesExSource for PagesExSrc {
    async fn offset2pages(&self, offset: u64, size: u64) -> Result<Vec<PageEx>, io::Error> {
        let pages_string = self.source.offset2pages_string(offset, size).await?;
        self.s2page.pages_string2pages(&pages_string).await
    }

    async fn offset2pages_filtered(
        &self,
        offset: u64,
        size: u64,
        f: &BasicFilter,
    ) -> Result<Vec<PageEx>, io::Error> {
        let pages_string = self.source.offset2pages_string(offset, size).await?;
        self.s2page
            .pages_string2pages_filtered(&pages_string, f)
            .await
    }
}

pub struct QueryEx {
    pub source: Arc<Box<dyn PagesExSource>>,
}

#[Object]
impl QueryEx {
    pub async fn pages(
        &self,
        offset: u64,
        size: u64,
        filter: Option<BasicFilter>,
    ) -> Result<Vec<PageEx>, io::Error> {
        match filter {
            None => self.source.offset2pages(offset, size).await,
            Some(f) => self.source.offset2pages_filtered(offset, size, &f).await,
        }
    }
}
