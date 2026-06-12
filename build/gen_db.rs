use std::{env, fs::File, io::BufWriter, io::Write, path::Path};

pub fn run() {
  let path = Path::new(&env::var("OUT_DIR").unwrap()).join("codegen_db.rs");
  let mut file = BufWriter::new(File::create(&path).unwrap());
  gen_essential(&mut file);
  gen_item_db(&mut file);
  gen_jinn_db(&mut file);
}

fn gen_essential(file: &mut BufWriter<File>) {
  // Language enum
  writeln!(
    file,
    r#"#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Language {{
  Japanese,
  English,
  ChineseTBS,
  ChineseTLA,
}}

impl Language {{
  pub const LANGUAGES: [Self; 4] = [Language::Japanese, Language::English, Language::ChineseTBS, Language::ChineseTLA];

  pub fn get_name(self) -> &'static str {{
    match self {{
      Language::Japanese => "日文",
      Language::English => "英文",
      Language::ChineseTBS => "1代新汉化",
      Language::ChineseTLA => "2代汉化",
    }}
  }}
}}

"#
  )
  .unwrap();

  // Struct
  writeln!(
    file,
    r"pub struct PreviewObject {{
  name: [&'static str; 4],
}}

impl PreviewObject {{
  pub fn get_name(&self, lang: Language) -> &'static str {{
    self.name[lang as usize]
  }}
}}

"
  )
  .unwrap();

  writeln!(file, "").unwrap();
}

fn gen_item_db(file: &mut BufWriter<File>) {
  let jp =
    serde_json::from_slice::<Vec<String>>(include_bytes!("../resources/json/item_name/jp.json"))
      .unwrap();

  let en =
    serde_json::from_slice::<Vec<String>>(include_bytes!("../resources/json/item_name/en.json"))
      .unwrap();

  let chs1 =
    serde_json::from_slice::<Vec<String>>(include_bytes!("../resources/json/item_name/chs1.json"))
      .unwrap();

  let chs2 =
    serde_json::from_slice::<Vec<String>>(include_bytes!("../resources/json/item_name/chs2.json"))
      .unwrap();

  let en_dyrati = serde_json::from_slice::<Vec<String>>(include_bytes!(
    "../resources/json/item_name/en_dyrati.json"
  ))
  .unwrap();

  writeln!(file, "pub const ITEM_NAMES: [PreviewObject; 460] = [").unwrap();

  for i in 0..460 {
    writeln!(
      file,
      "{}",
      format!(
        "  PreviewObject {{ name: [\"{}\", \"{}\", \"{}\", \"{}\"] }},",
        jp[i], en[i], chs1[i], chs2[i]
      )
    )
    .unwrap();
  }

  writeln!(file, "];\n").unwrap();

  writeln!(file, "pub const DYRATI_ITEM_NAMES: [&str; 460] = [").unwrap();
  for i in 0..460 {
    writeln!(file, "  \"{}\",", en_dyrati[i]).unwrap();
  }
  writeln!(file, "];\n").unwrap();
}

fn gen_jinn_db(file: &mut BufWriter<File>) {
  let jp =
    serde_json::from_slice::<Vec<String>>(include_bytes!("../resources/json/djinn_name/jp.json"))
      .unwrap();

  let en =
    serde_json::from_slice::<Vec<String>>(include_bytes!("../resources/json/djinn_name/en.json"))
      .unwrap();

  let chs1 =
    serde_json::from_slice::<Vec<String>>(include_bytes!("../resources/json/djinn_name/chs1.json"))
      .unwrap();

  let chs2 =
    serde_json::from_slice::<Vec<String>>(include_bytes!("../resources/json/djinn_name/chs2.json"))
      .unwrap();

  writeln!(
    file,
    "pub const ALL_DJINN_NAMES: [[PreviewObject; 7]; 4] = ["
  )
  .unwrap();

  let gameplay_order = [0, 2, 3, 1];

  for i in 0..4 {
    writeln!(file, "  [").unwrap();
    for j in gameplay_order[i] * 7..(gameplay_order[i] + 1) * 7 {
      writeln!(
        file,
        "{}",
        format!(
          "    PreviewObject {{ name: [\"{}\", \"{}\", \"{}\", \"{}\"] }},",
          jp[j], en[j], chs1[j], chs2[j]
        )
      )
      .unwrap();
    }
    writeln!(file, "  ],").unwrap();
  }
  writeln!(file, "];\n").unwrap();
}
