use std::path::Path;
use anyhow::anyhow;
use yaml_rust2::{Yaml, YamlEmitter, YamlLoader};
//--------------------------------------------------------------------------------------------------



pub fn add_yaml_to_string(yaml: &Yaml, out_str: &mut String) -> anyhow::Result<()> {
    {
        let mut emitter = YamlEmitter::new(out_str);
        emitter.dump(yaml) ?; // dump the YAML object to a String
    } // !!! in {} block according to official example
    Ok(())
}


pub fn yaml_to_string(yaml: &Yaml) -> anyhow::Result<String> {
    let mut out_str = String::new();
    add_yaml_to_string(yaml, &mut out_str) ?;
    Ok(out_str)
}



pub fn save_yaml(yaml_docs: &Vec<Yaml>, to_file: &Path) -> anyhow::Result<()> {
    let mut out_str = String::new();

    for ref yaml in yaml_docs {
        add_yaml_to_string(yaml, &mut out_str) ?;
        out_str.push('\n');
    }

    out_str.push_str("\n\n");
    let _ = std::fs::write(to_file, out_str) ?;
    Ok(())
}


pub fn load_yaml(yaml_file: &Path) -> anyhow::Result<Vec<Yaml>> {

    let yaml_str = std::fs::read_to_string(yaml_file)
        .map_err(|err| anyhow!("Error of opening [{yaml_file:?}] ({err:?})")) ?;

    // Multi document support, doc is a yaml::Yaml
    let yaml_docs = YamlLoader::load_from_str(&yaml_str) ?;
    Ok(yaml_docs)
}
