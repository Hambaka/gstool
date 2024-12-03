use std::path::PathBuf;
use clap::{arg, ArgAction, ArgGroup, Command, crate_version, value_parser};
use clap::ValueHint::{FilePath, Other};
use klask::Localization;
use crate::util::enums::{CheatVersion, PasswordGrade, PasswordVersion, NameType, BuildDateType};

pub fn build_cli() -> Command {
  Command::new("gstool")
    .version(crate_version!())
    .author("Hambaka")
    .about("版本：v0.1.0　作者：Hambaka　描述：为 GBA 的《黄金太阳》系列游戏准备的小工具")
    .propagate_version(true)
    .subcommand_required(true)
    .arg_required_else_help(true)
    .subcommand(Command::new("（1）密码导出功能")
      .about(
        "使用该功能可导出密码，可导出以下几类文件：\n\
        （1）密码的文本文件（支持导出日文版/英文版密码）\n\
        （2）密码的内存转储文件\n\
        （3）密码的金手指文本文件，可以在《黄金太阳 失落的时代》中使用（支持日/美/德/西/法/意版本）\n\
        （4）密码数据的文本文件，可结合 Dyrati 的“Golden Sun Password Generator”表格使用\n\n\
        注：Mobile&Elffinal/SOMA 的《黄金太阳 失落的时代》汉化版基于美版制作，因此金手指版本选择美版即可。")
      .subcommand(
        Command::new("（1.1）使用存档文件")
          .about(
            "“生成内存转储文件”，“生成密码金手指”，“导出密码数据至文本文件”。\n\
            程序默认密码级别为“金”，密码文本版本为“英文”，因此以上三项都是可以留空或者不勾选的可选选项。")
          .args(&[
            arg!(<输入文件路径> "《黄金太阳 开启的封印》的存档文件。").value_parser(value_parser!(PathBuf)).required(true).value_hint(FilePath),
            arg!(--"密码级别" <VALUE> "目标密码级别，若留空则自动指定默认值“金”。").value_parser(clap::builder::EnumValueParser::<PasswordGrade>::new()).default_value("金"),
            arg!(--"密码文本版本" <VALUE> "生成指定版本的密码文本文件，若留空则自动指定默认值“英文”。").value_parser(clap::builder::EnumValueParser::<PasswordVersion>::new()).default_value("英文"),
            arg!(--"导出所有存档" "若不勾选，则仅导出通关存档的相关文件。").action(ArgAction::SetTrue),
            arg!(--"生成内存转储文件" "若勾选，则生成密码的内存转储文件。").action(ArgAction::SetTrue),
            arg!(--"生成密码金手指" <VALUE> "若选择了值，则生成指定版本的金手指文本文件。").value_parser(clap::builder::EnumValueParser::<CheatVersion>::new()),
            arg!(--"导出密码数据至文本文件" "若勾选，则导出适用于 Dyrati 的“Golden Sun Password Generator”表格的密码数据文本文件。").action(ArgAction::SetTrue),
            arg!(--"指定输出文件保存目录" <OUTPUT_DIR> "自定义输出文件夹（推荐留空），若留空则在输入文件的同目录下创建文件夹保存生成的所有文件。").value_parser(value_parser!(PathBuf)).value_hint(Other)
          ])
          .group(ArgGroup::new("sav_args")
            .args(["生成内存转储文件", "生成密码金手指", "导出密码数据至文本文件"])
            .required(false)
            .multiple(true)
          )
      )
      .subcommand(
        Command::new("（1.2）使用密码文本文件")
          .about(
            "“密码级别”，“转换密码版本”，“生成内存转储文件”，“生成密码金手指”，“导出密码数据至文本文件”。\n\
            以上五项虽然都是可以留空或者不勾选的可选选项，但是必须要有其中一项有选择值或者已勾选。")
          .args(&[
            arg!(<输入文件路径> "黄金太阳的密码文本文件。").value_parser(value_parser!(PathBuf)).required(true).value_hint(FilePath),
            arg!(--"密码级别" <VALUE> "目标密码级别（仅用于降级），指定级别不可高于输入文件的级别。若两者级别相同则忽略；若可降级则生成降级后的密码文本文件（及相关文件）；若留空，则目标级别同输入文件。").value_parser(clap::builder::EnumValueParser::<PasswordGrade>::new()),
            arg!(--"转换密码版本" "若勾选，则自动检测输入的密码文本的版本，并转换输出另外一个版本的密码文本文件，即日转英，英转日。").action(ArgAction::SetTrue),
            arg!(--"生成内存转储文件" "若勾选，则生成密码的内存转储文件").action(ArgAction::SetTrue),
            arg!(--"生成密码金手指" <VALUE> "若选择了值，则生成指定版本的金手指文本文件").value_parser(clap::builder::EnumValueParser::<CheatVersion>::new()),
            arg!(--"导出密码数据至文本文件" "若勾选，则导出适用于 Dyrati 的“Golden Sun Password Generator”表格的密码数据文本文件").action(ArgAction::SetTrue),
            arg!(--"指定输出文件保存目录" <OUTPUT_DIR> "自定义输出文件夹（推荐留空），若留空则在输入文件的同目录下创建文件夹保存生成的所有文件。").value_parser(value_parser!(PathBuf)).value_hint(Other)
          ])
          .group(ArgGroup::new("txt_args")
            .args(["密码级别", "转换密码版本", "生成内存转储文件", "生成密码金手指", "导出密码数据至文本文件"])
            .required(true)
            .multiple(true)
          )
      )
      .subcommand(
        Command::new("（1.3）使用内存转储文件")
          .about(
            "“密码级别”，“密码文本版本”，“生成密码金手指”，“导出密码数据至文本文件”。\n\
            以上四项虽然都是可以留空或者不勾选的可选选项，但是必须要有其中一项有选择值或者已勾选。")
          .args(&[
            arg!(<输入文件路径> "黄金太阳的密码内存转储文件").value_parser(value_parser!(PathBuf)).required(true).value_hint(FilePath),
            arg!(--"密码级别" <VALUE> "目标密码级别（仅用于降级），指定级别不可高于输入文件的级别。若两者级别相同则忽略；若可降级则生成降级后的密码文本文件（及相关文件）；若留空，则目标级别同输入文件。").value_parser(clap::builder::EnumValueParser::<PasswordGrade>::new()),
            arg!(--"密码文本版本" <VALUE> "若选择了值，则生成指定版本的密码文本文件。").value_parser(clap::builder::EnumValueParser::<PasswordVersion>::new()),
            arg!(--"生成密码金手指" <VALUE> "若选择了值，则生成指定版本的金手指文本文件。").value_parser(clap::builder::EnumValueParser::<CheatVersion>::new()),
            arg!(--"导出密码数据至文本文件" "若勾选，则导出适用于 Dyrati 的“Golden Sun Password Generator”表格的密码数据文本文件。").action(ArgAction::SetTrue),
            arg!(--"指定输出文件保存目录" <OUTPUT_DIR> "自定义输出文件夹（推荐留空），若留空则在输入文件的同目录下创建文件夹保存生成的所有文件。").value_parser(value_parser!(PathBuf)).value_hint(Other)
          ])
          .group(ArgGroup::new("dmp_args")
            .args(["密码级别", "密码文本版本", "生成密码金手指", "导出密码数据至文本文件"])
            .required(true)
            .multiple(true)
          )
      )
    )
    .subcommand(
      Command::new("（2）存档转换功能")
        .about(
          "使用该功能可转换《黄金太阳 开启的封印》或《黄金太阳 失落的时代》存档的版本：\n\
          （1）修改主角团所有成员的名字为指定语言版本的默认名\n\
          （2）通过修改存档中的游戏构建日期来转换存档版本\n\n\
          注：\n\
          （1）如果存档中的构建日期与游戏 ROM 中的构建日期不匹配，游戏将强制玩家从神殿开始游戏。\n\
          （2）Mobile&Elffinal/SOMA 的《黄金太阳 失落的时代》汉化版基于美版制作，所以 ROM 中的构建日期同美版。\n\
          （3）《黄金太阳 开启的封印》新汉化版基于日版制作，所以 ROM 中的构建日期同日版。")
        .args(&[
          arg!(<输入文件路径> "《黄金太阳 开启的封印》或《黄金太阳 失落的时代》的存档文件").value_parser(value_parser!(PathBuf)).required(true).value_hint(FilePath),
          arg!(--"名字版本" <VALUE> "若选择了值，则将存档中的主角团所有成员的名字转换至指定版本的默认名；若留空则不修改。").value_parser(clap::builder::EnumValueParser::<NameType>::new()),
          arg!(--"构建日期版本" <VALUE> "若选择了值，则将存档中的构建日期转换至指定版本的构建日期；若留空则不修改。").value_parser(clap::builder::EnumValueParser::<BuildDateType>::new()),
          arg!(--"指定输出文件保存路径" <OUTPUT_FILE> "自定义输出的存档文件路径（推荐留空），若留空则将转换后的存档文件保存到输入文件的同目录下。").value_parser(value_parser!(PathBuf)).value_hint(Other)
        ])
        .group(ArgGroup::new("conv_args")
          .args(["名字版本", "构建日期版本"])
          .required(true)
          .multiple(true)
        )
    )
    .subcommand(
      Command::new("（3）自定义角色名")
        .about(
          "该功能仅支持《黄金太阳 开启的封印》新汉化版，主要面向自定义中文名（但也支持中日英混合）。\n\
          请在输入框中输入至少一个角色的名字，再点击“运行”即可在下方的输出区域展示如何输入自定义名。\n\n\
          注：\n\
          （1）强烈推荐在“运行”后点击“Copy Output”按钮，输出内容将会复制到剪贴板，之后便可粘贴到记事本进行查看。\n\
          （2）若想命名初代主角团其他的三位成员，请在起名界面连按三下“Select”键，成功后可听到提示音。\n\
          （3）若想命名最后的三位角色，请接着上条继续输入↑↓↑↓←→←→↑→↓←↑和“Select”，成功后可听到提示音。")
        .args(&[
          arg!(--"罗宾" <VALUE> "罗宾的自定义角色名。").value_parser(clap::builder::StringValueParser::new()),
          arg!(--"杰拉德" <VALUE> "杰拉德的自定义角色名。").value_parser(clap::builder::StringValueParser::new()),
          arg!(--"伊万" <VALUE> "伊万的自定义角色名。").value_parser(clap::builder::StringValueParser::new()),
          arg!(--"梅雅莉" <VALUE> "梅雅莉的自定义角色名。").value_parser(clap::builder::StringValueParser::new()),
          arg!(--"加西亚" <VALUE> "加西亚的自定义角色名。").value_parser(clap::builder::StringValueParser::new()),
          arg!(--"加斯敏" <VALUE> "加斯敏的自定义角色名。").value_parser(clap::builder::StringValueParser::new()),
          arg!(--"西芭" <VALUE> "西芭的自定义角色名。").value_parser(clap::builder::StringValueParser::new()),
        ])
        .group(ArgGroup::new("rename_args")
          .args(["罗宾", "杰拉德", "伊万", "梅雅莉", "加西亚", "加斯敏", "西芭"])
          .required(true)
          .multiple(true)
        )
    )
}

pub fn zh_hans_localization() -> Localization {
  let mut loc = Localization::default();
  loc.optional = "（非必要/可选）".into();
  loc.select_file = "选择文件……".into();
  loc.select_directory = "选择目录……".into();
  loc.new_value = "新值".into();
  loc.reset = "重置".into();
  loc.reset_to_default = "重置为默认值".into();
  loc.error_is_required = ("缺少 '".into(), "' 参数".into());
  loc.arguments = "参数".into();
  loc.env_variables = "环境变量".into();
  loc.error_env_var_cant_be_empty = "环境变量不得为空".into();
  loc.input = "输入".into();
  loc.text = "文本".into();
  loc.file = "文件".into();
  loc.working_directory = "工作目录".into();
  loc.run = "运行".into();
  loc.kill = "终止".into();
  loc.running = "运行中".into();
  loc
}
