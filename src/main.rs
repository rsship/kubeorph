#![allow(dead_code)]

mod size;

use clap::Parser;
use kube::api::ListParams;
use kube::api::WatchParams;
use kube::core::DynamicObject;
use kube::discovery::ApiCapabilities;
use kube::discovery::ApiResource;
use kube::discovery::Discovery;
use kube::discovery::Scope;
use kube::Api;
use kube::Client;
use kube::ResourceExt;
use size::terminal_size;
use std::fmt::Formatter;

use anyhow::Result;

const CURRENT_TAB: u16 = 6;

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
                let width = result
                    .iter()
                    .map(|x| {
                        if let Some(name) = &x.metadata.name {
                            return name.len();
                        } else {
                            return 69;
                        }
                    })
                    .max()
                    .unwrap_or(69);

                // NAMESPACE            NAME                                         READY   STATUS    RESTARTS   AGE
                pretty_print(&result);
            }
        }

        Ok(())
    }

    async fn watch(&self, api: Api<DynamicObject>, wc: WatchParams) -> Result<()> {
        let mut stream = api.watch(&wc, "0").await?.boxed();

        Ok(())
    }

    async fn edit(&self, api: Api<DynamicObject>) -> Result<()> {
        todo!("Not implemented yet");
    }

    async fn apply(&self, api: Api<DynamicObject>) -> Result<()> {
        todo!("Not implemeted yet");
    }

    async fn delete(&self, api: Api<DynamicObject>) -> Result<()> {
        todo!("Not implemented yet");
    }

    // Get,
    // Delete,
    // Edit,
    // Watch,
    // Apply,
}

#[tokio::main]
async fn main() -> Result<()> {
    let app: App = clap::Parser::parse();

    let client = Client::try_default().await?;

    let discovery = Discovery::new(client.clone()).run().await?;
    if let Some(resource) = &app.resource {
        let (ar, caps) =
            resolve_resource_by_name(&discovery, resource).expect("No resources found");

        let mut lp = ListParams::default();
        if let Some(label) = &app.selector {
            lp = lp.labels(label);
        }

        let mut wc = WatchParams::default();
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
                app.watch(dynapi, wc).await?;
            }
            Verb::Delete => {
                app.delete(dynapi).await?;
            }

            Verb::Edit => {
                app.edit(dynapi).await?;
            }

            Verb::Apply => {
                app.apply(dynapi).await?;
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

fn pretty_print(data: &Vec<DynamicObject>) {
    let (_, height) = terminal_size().unwrap_or((30, 180));

    let each_size = (height / CURRENT_TAB) as usize;

    println!(
        "{:<each_size$} {:<each_size$} {:<each_size$} {:<each_size$} {:<each_size$} {:<each_size$}",
        "NAMESPACE", "NAME", "READY", "STATUS", "RESTARTS", "AGE"
    );

    for inst in data {
        let mut status = String::from("None");

        if let Some(s) = inst.data.get("status") {
            if let Some(phase) = s.get("phase") {
                status = phase.to_string();
            }
        }

        println!(
        "{:<each_size$} {:<each_size$} {:<each_size$} {:<each_size$} {:<each_size$} {:<each_size$}",
        inst.namespace().unwrap() , inst.name_any(), "READY", status, "RESTARTS", "AGE"
        );
        break;
    }
}

fn calculate_age() {
    todo!()
}
