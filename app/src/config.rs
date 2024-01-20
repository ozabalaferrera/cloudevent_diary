use envconfig::Envconfig;

#[derive(Envconfig)]
pub struct AppConfig {
    pub db_host: String,
    pub db_name: String,
    pub db_schema: String,
    pub db_table: String,
    pub db_user: String,
    pub db_password: String,
    #[envconfig(default = "10")]
    pub db_pool_size: u32,
    #[envconfig(default = "[helm release name]")]
    pub helm_release_name: String,
    #[envconfig(default = "[helm releae revision]")]
    pub helm_release_revision: String,
    #[envconfig(default = "[helm chart name]")]
    pub helm_chart_name: String,
    #[envconfig(default = "[helm chart version]")]
    pub helm_chart_version: String,
    #[envconfig(default = "[helm release namespace]")]
    pub helm_release_namespace: String,
    #[envconfig(default = "[hostname]")]
    pub hostname: String,
    #[envconfig(default = "8080")]
    pub web_port: u16,
}
