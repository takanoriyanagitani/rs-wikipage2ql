use std::io;
use std::process::ExitCode;
use std::sync::Arc;

use tokio::net::TcpListener;

use async_graphql_axum::GraphQLRequest;
use async_graphql_axum::GraphQLResponse;

use rs_wikipage2ql::async_graphql;
use rs_wikipage2ql::async_trait;
use rs_wikipage2ql::rs_wikipages2struct;

use async_graphql::EmptyMutation;
use async_graphql::EmptySubscription;
use async_graphql::Schema;

use rs_wikipages2struct::Page;

use rs_wikipage2ql::PagesExSource;
use rs_wikipage2ql::PagesExSrc;
use rs_wikipage2ql::PagesStringSource;
use rs_wikipage2ql::PagesStringToPages;
use rs_wikipage2ql::QueryEx;

fn env2addr_port() -> Result<String, io::Error> {
    std::env::var("ENV_ADDR_PORT").map_err(io::Error::other)
}

fn env2bz_filename() -> Result<String, io::Error> {
    std::env::var("ENV_BZIP_FILENAME").map_err(io::Error::other)
}

type PagesExSchema = Schema<QueryEx, EmptyMutation, EmptySubscription>;

async fn req2res_ex(s: &PagesExSchema, req: GraphQLRequest) -> GraphQLResponse {
    s.execute(req.into_inner()).await.into()
}

struct PagesStrToPages {}

#[async_trait::async_trait]
impl PagesStringToPages for PagesStrToPages {
    async fn pages_string2pages(&self, pages: &str) -> Result<Vec<Page>, io::Error> {
        rs_wikipages2struct::xmlpages2pages(pages)
    }
}

struct PagesStrSrc {
    bzfilename: String,
}

#[async_trait::async_trait]
impl PagesStringSource for PagesStrSrc {
    async fn offset2pages_string(&self, offset: u64, size: u64) -> Result<String, io::Error> {
        let mut buf: String = String::new();
        rs_wikibzip2pages::filepath2pages(&self.bzfilename, offset, size, &mut buf)?;
        Ok(buf)
    }
}

async fn sub() -> Result<(), io::Error> {
    let bzfn: String = env2bz_filename()?;
    let ps2 = PagesStrSrc { bzfilename: bzfn };

    let pstp = PagesStrToPages {};

    let pes = PagesExSrc {
        source: Arc::new(Box::new(ps2)),
        s2page: Arc::new(Box::new(pstp)),
    };

    let abpes: Arc<Box<dyn PagesExSource>> = Arc::new(Box::new(pes));
    let qex = QueryEx { source: abpes };

    let sch_ex = Schema::new(qex, EmptyMutation, EmptySubscription);
    let sdl: String = sch_ex.sdl();
    std::fs::write("./wikipage2ql.gql", sdl.as_bytes())?;

    let addr_port: String = env2addr_port()?;

    let lis = TcpListener::bind(addr_port).await?;

    let app = axum::Router::new().route(
        "/",
        axum::routing::post(|req| async move { req2res_ex(&sch_ex, req).await }),
    );

    axum::serve(lis, app).await
}

#[tokio::main]
async fn main() -> ExitCode {
    match sub().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}
