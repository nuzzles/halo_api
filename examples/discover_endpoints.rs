//! Prints the endpoint inventory advertised by Halo Infinite's live HIPC manifest.

const MANIFEST_URL: &str =
    "https://settings.svc.halowaypoint.com/settings/hipc/e2a0a7c6-6efe-42af-9283-c2ab73250c48";

fn child_text<'a, 'input>(node: roxmltree::Node<'a, 'input>, name: &str) -> &'a str {
    node.children()
        .find(|child| child.is_element() && child.tag_name().name() == name)
        .and_then(|child| child.text())
        .unwrap_or("")
}

fn descendant_text<'a, 'input>(node: roxmltree::Node<'a, 'input>, name: &str) -> &'a str {
    node.descendants()
        .find(|child| child.is_element() && child.tag_name().name() == name)
        .and_then(|child| child.text())
        .unwrap_or("")
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let xml = reqwest::get(MANIFEST_URL)
        .await?
        .error_for_status()?
        .text()
        .await?;
    let document = roxmltree::Document::parse(&xml)?;

    let endpoints = document
        .descendants()
        .find(|node| node.has_tag_name("Endpoints"))
        .ok_or("manifest has no Endpoints section")?;

    println!("NAME\tAUTHORITY\tCLEARANCE\tPATH\tQUERY");
    for entry in endpoints.children().filter(|node| node.is_element()) {
        let key = child_text(entry, "Key");
        let value = entry
            .children()
            .find(|child| child.is_element() && child.tag_name().name() == "Value")
            .ok_or("endpoint entry has no Value")?;
        println!(
            "{}\t{}\t{}\t{}\t{}",
            key,
            descendant_text(value, "AuthorityId"),
            descendant_text(value, "ClearanceAware"),
            descendant_text(value, "Path"),
            descendant_text(value, "QueryString"),
        );
    }
    Ok(())
}
