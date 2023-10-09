use clap::Parser;
use kube::api::ListParams;
use kube::core::DynamicObject;
use kube::discovery::ApiCapabilities;
use kube::discovery::ApiResource;
use kube::discovery::Discovery;
use kube::discovery::Scope;
use kube::runtime::watcher;
use kube::Api;
use kube::Client;
use kube::ResourceExt;
use std::fmt::Formatter;

use anyhow::Result;

#[derive(clap::ValueEnum, Clone, Debug)]
enum OutputMode {
    Pretty,
    Yaml,
}

#[derive(Debug, Clone, clap::ValueEnum)]
enum Verb {
    Get,
    Delete,
    Edit,
    Watch,
    Apply,
}

impl std::fmt::Display for OutputMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        f.pad(match self {
            Self::Pretty => "pretty",
            Self::Yaml => "yaml",
        })
    }
}
#[derive(Parser, Debug)]
struct App {
    #[arg(long, short, default_value_t= OutputMode::Pretty )]
    output: OutputMode,
    #[arg(long, short)]
    namespace: Option<String>,
    #[arg(long, short = 'l')]
    selector: Option<String>,
    #[arg(long, short = 'A')]
    all: bool,
    verb: Verb,
    resource: Option<String>,
    name: Option<String>,
}

impl App {
    async fn get(&self, api: Api<DynamicObject>, lp: ListParams) -> Result<()> {
        let mut result = if let Some(n) = &self.name {
            vec![api.get(n).await?]
        } else {
            api.list(&lp).await?.items
        };

        //NOTE: search managed fileds in k8s;
        result
            .iter_mut()
            .for_each(|obj| obj.managed_fields_mut().clear());

        match self.output {
            OutputMode::Yaml => {}
            OutputMode::Pretty => {
                // let max_name_len = result
                //     .iter()
                //     .map(|x| {
                //         if let Some(name) = &x.metadata.name {
                //             return name.len();
                //         } else {
                //             return 69;
                //         }
                //     })
                //     .max()
                //     .unwrap_or(69);
                println!("{} {}", "NAME", "AGE");
                for inst in result {
                    let age = inst.creation_timestamp().map(|x| x.0).unwrap_or_default();
                    let name = inst.metadata.name.unwrap_or("".to_string());
                    println!("{} -> {:?}", name, age);
                }
            }
        }

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let app: App = clap::Parser::parse();
    println!("{:?}", app);

    let client = Client::try_default().await?;

    let discovery = Discovery::new(client.clone()).run().await?;
    if let Some(resource) = &app.resource {
        let (ar, caps) =
            resolve_resource_by_name(&discovery, resource).expect("No resources found");

        let mut lp = ListParams::default();
        if let Some(label) = &app.selector {
            lp = lp.labels(label);
        }

        let mut wc = watcher::Config::default();
        if let Some(label) = &app.selector {
            wc = wc.labels(label);
        }

        let dynapi = if caps.scope == Scope::Cluster || app.all {
            Api::<DynamicObject>::all_with(client, &ar)
        } else if let Some(namespace) = &app.namespace {
            Api::<DynamicObject>::namespaced_with(client, &namespace, &ar)
        } else {
            Api::<DynamicObject>::default_namespaced_with(client, &ar)
        };

        match app.verb {
            Verb::Get => {
                app.get(dynapi, lp).await?;
            }

            Verb::Watch => {
                todo!();
            }
            Verb::Delete => {
                todo!();
            }

            Verb::Edit => {
                todo!();
            }

            Verb::Apply => {
                todo!();
            }

            _ => {
                todo!();
            }
        }
    }

    Ok(())
}

fn resolve_resource_by_name(
    discovery: &Discovery,
    name: &str,
) -> Option<(ApiResource, ApiCapabilities)> {
    discovery
        .groups()
        .flat_map(|group| {
            group
                .resources_by_stability()
                .into_iter()
                .map(move |res| (group, res))
        })
        .filter(|(_, (res, _))| {
            name.eq_ignore_ascii_case(&res.kind) || name.eq_ignore_ascii_case(&res.plural)
        })
        .min_by_key(|(group, _)| group.name())
        .map(|(_, res)| res)
}

fn resolve_resource_by_name2(
    discovery: &Discovery,
    name: &str,
) -> Option<(ApiResource, ApiCapabilities)> {
    for group in discovery.groups() {
        for (ar, caps) in group.resources_by_stability() {
            if name.eq_ignore_ascii_case(&ar.kind) || name.eq_ignore_ascii_case(&ar.plural) {
                return Some((ar, caps));
            }
        }
    }

    None
}
