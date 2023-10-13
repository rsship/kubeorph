use super::size::terminal_size;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::Time;
use kube::core::DynamicObject;
use kube::ResourceExt;

const CURRENT_TAB_SIZE: u16 = 6;

pub fn pretty_print(data: Vec<DynamicObject>) {
    let (_, height) = terminal_size().unwrap_or((30, 180));

    let each_size = (height / CURRENT_TAB_SIZE) as usize;

    let max_width = data
        .iter()
        .map(|x| x.metadata.name.as_ref().unwrap().len())
        .max()
        .unwrap_or(each_size);

    //NOTE: commented for the future use
    //
    // println!(
    //     "{:width$} {} {} {} {} {}",
    //     "NAMESPACE",
    //     "NAME",
    //     "READY",
    //     "STATUS",
    //     "RESTARTS",
    //     "AGE",
    //     width = max_width
    // );
    //

    let ns_max_len = data
        .iter()
        .map(|x| x.namespace().unwrap().len())
        .max()
        .unwrap_or(69);

    let pod_max_len = data.iter().map(|x| x.name_any().len()).max().unwrap();

    for inst in data {
        let mut status = String::from("None");
        let mut cond = String::from("None");
        if let Some(s) = inst.data.get("status") {
            if let Some(phase) = s.get("phase") {
                status = phase.to_string();
            }

            if let Some(conditions) = s.get("conditions") {
                cond = conditions[0].to_string();
            }
        }

        let age = time_to_age(inst.creation_timestamp().unwrap());

        println!(
            "{:ns_len$} {:name_len$} {} {} {} {}",
            inst.namespace().unwrap(),
            inst.name_any(),
            "READY",
            "STATUS",
            "RESTARTS",
            "AGE",
            ns_len = ns_max_len,
            name_len = pod_max_len,
        );
    }
}

fn time_to_age(t: Time) -> String {
    t.0.to_string()
}
