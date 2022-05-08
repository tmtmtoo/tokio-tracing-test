type BoxedError = Box<dyn std::error::Error + Send + Sync + 'static>;

struct HttpClient(reqwest::Client);

impl HttpClient {
    #[tracing::instrument]
    fn new() -> anyhow::Result<Self> {
        reqwest::ClientBuilder::new()
            .connect_timeout(std::time::Duration::from_secs(1))
            .build()
            .map(Self)
            .map_err(Into::into)
    }

    #[tracing::instrument(ret, err)]
    fn to_json(text: &str) -> Result<serde_json::Value, BoxedError> {
        serde_json::from_str(text).map_err(Into::into)
    }

    #[tracing::instrument(skip(self), ret, err)]
    async fn get(&self, url: &str) -> Result<serde_json::Value, BoxedError> {
        let resp = self.0.get(url).send().await?;
        let text = resp.text().await?;
        let json = Self::to_json(&text)?;
        Ok(json)
    }
}

#[tracing::instrument(
    ret(Display),
    err,
    fields(extras = url.chars().rev().collect::<String>().as_str())
)]
async fn get_json_with_tracing(url: &str) -> Result<serde_json::Value, BoxedError> {
    let client = HttpClient::new()?;
    let response = client.get(url).await?;
    Ok(response)
}

#[tokio::main]
async fn main() -> Result<(), BoxedError> {
    use opentelemetry::sdk::{
        trace::{self, IdGenerator, Sampler},
        Resource,
    };
    use opentelemetry::KeyValue;
    use opentelemetry_otlp::WithExportConfig;
    use tracing_subscriber::prelude::*;

    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint("http://otel:4317"),
        )
        .with_trace_config(
            trace::config()
                .with_sampler(Sampler::AlwaysOn)
                .with_id_generator(IdGenerator::default())
                .with_resource(Resource::new(vec![KeyValue::new(
                    "service.name",
                    "example",
                )])),
        )
        .install_batch(opentelemetry::runtime::Tokio)?;

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::Layer::new().json())
        .with(tracing_opentelemetry::layer().with_tracer(tracer))
        .try_init()?;

    let _ = get_json_with_tracing("https://httpbin.org/get").await;

    opentelemetry::global::shutdown_tracer_provider();

    Ok(())
}
