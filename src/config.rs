use envconfig::Envconfig;

#[derive(Envconfig)]
pub struct Config {
  #[envconfig(from = "HOST", default = "127.0.0.1")]
  pub host: String,

  #[envconfig(from = "PORT", default = "8080")]
  pub port: u16,
}
