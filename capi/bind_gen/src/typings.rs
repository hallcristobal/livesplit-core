use heck::MixedCase;
use std::collections::BTreeMap;
use std::io::{Result, Write};
use {typescript, Class, Function, Type, TypeKind};

fn get_hl_type_with_null(ty: &Type) -> String {
    let mut formatted = get_hl_type_without_null(ty);
    if ty.is_nullable {
        formatted.push_str(" | null");
    }
    formatted
}

fn get_hl_type_without_null(ty: &Type) -> String {
    if ty.is_custom {
        match ty.kind {
            TypeKind::Ref => format!("{}Ref", ty.name),
            TypeKind::RefMut => format!("{}RefMut", ty.name),
            TypeKind::Value => ty.name.clone(),
        }
    } else {
        match (ty.kind, ty.name.as_str()) {
            (TypeKind::Ref, "c_char") => "string",
            (_, t) if !ty.is_custom => match t {
                "i8" => "number",
                "i16" => "number",
                "i32" => "number",
                "i64" => "number",
                "u8" => "number",
                "u16" => "number",
                "u32" => "number",
                "u64" => "number",
                "usize" => "number",
                "f32" => "number",
                "f64" => "number",
                "bool" => "boolean",
                "()" => "void",
                "c_char" => "string",
                "Json" => "any",
                x => x,
            },
            _ => unreachable!(),
        }.to_string()
    }
}

fn write_class_comments<W: Write>(mut writer: W, comments: &[String]) -> Result<()> {
    write!(
        writer,
        r#"
/**"#
    )?;

    for comment in comments {
        write!(
            writer,
            r#"
 * {}"#,
            comment
                .replace("<NULL>", "null")
                .replace("<TRUE>", "true")
                .replace("<FALSE>", "false")
        )?;
    }

    write!(
        writer,
        r#"
 */"#
    )
}

fn write_fn<W: Write>(mut writer: W, function: &Function) -> Result<()> {
    let is_static = function.is_static();
    let has_return_type = function.has_return_type();
    let return_type_with_null = get_hl_type_with_null(&function.output);
    let method = function.method.to_mixed_case();

    if !function.comments.is_empty() {
        write!(
            writer,
            r#"
    /**"#
        )?;

        for comment in &function.comments {
            write!(
                writer,
                r#"
     * {}"#,
                comment
                    .replace("<NULL>", "null")
                    .replace("<TRUE>", "true")
                    .replace("<FALSE>", "false")
            )?;
        }

            write!(
                writer,
                r#"
     */"#
            )?;
    }

    if method == "new" {
        write!(
            writer,
            r#"
    {}("#,
            "constructor"
        )?;
    } else {
        write!(
            writer,
            r#"
    {}("#,
            method
        )?;
    }

    for (i, &(ref name, ref ty)) in function
        .inputs
        .iter()
        .skip(if is_static { 0 } else { 1 })
        .enumerate()
    {
        if i != 0 {
            write!(writer, ", ")?;
        }
        write!(writer, "{}", name.to_mixed_case())?;
            write!(writer, ": {}", get_hl_type_with_null(ty))?;
    }

    if has_return_type {
        write!(
            writer,
            r#"): {};
        "#,
            return_type_with_null
        )?;
    } else {
        write!(
            writer,
            r#"): void;
        "#
        )?;
    }

    Ok(())
}

pub fn write<W: Write>(
    mut writer: W,
    classes: &BTreeMap<String, Class>,
) -> Result<()> {
    write!(
        writer,
        r#"declare module "livesplit-core" {{"#
    )?;
        write!(
            writer,
            r#"
{}"#,
            typescript::HEADER
        )?;

    for (class_name, class) in classes {
        let class_name_ref = format!("{}Ref", class_name);
        let class_name_ref_mut = format!("{}RefMut", class_name);

        write_class_comments(&mut writer, &class.comments)?;

        write!(
            writer,
            r#"
class {class} {{"#,
            class = class_name_ref
        )?;

        for function in &class.shared_fns {
            write_fn(&mut writer, function)?;
        }

        if class_name == "SharedTimer" {
                write!(
                    writer,
                    "{}",
                    r#"
    readWith<T>(action: (timer: TimerRef) => T): T;
    writeWith<T>(action: (timer: TimerRefMut) => T): T;
	"#
                )?;
        }

            write!(
                writer,
                r#"
    constructor(ptr: number);"#
            )?;

		write!(
			writer,
			r#"
}}"#
		)?;

        write_class_comments(&mut writer, &class.comments)?;

        write!(
            writer,
            r#"
class {class} extends {base_class} {{"#,
            class = class_name_ref_mut,
            base_class = class_name_ref
        )?;

        for function in &class.mut_fns {
            write_fn(&mut writer, function)?;
        }

        if class_name == "RunEditor" {
                write!(
                    writer,
                    "{}",
                    r#"
    setGameIconFromArray(data: Int8Array): void;
    activeSetIconFromArray(data: Int8Array): void;
	"#
                )?;
        }

        write!(
            writer,
            r#"
}}
"#
        )?;

        write_class_comments(&mut writer, &class.comments)?;

        write!(
            writer,
            r#"
class {class} extends {base_class} {{"#,
            class = class_name,
            base_class = class_name_ref_mut
        )?;
            write!(
                writer,
                r#"
    with<T>(closure: (obj: {class}) => T): T;
    dispose();"#,
                class = class_name
            )?;

        for function in class.static_fns.iter().chain(class.own_fns.iter()) {
            if function.method != "drop" {
                write_fn(&mut writer, function)?;
            }
        }

        if class_name == "Run" {
                write!(
                    writer,
                    "{}",
                    r#"
    static parseArray(data: Int8Array, path: string, loadFiles: boolean): ParseRunResult;
    static parseFile(file: any, path: string, loadFiles: boolean): ParseRunResult;
    static parseString(text: string, path: string, loadFiles: boolean): ParseRunResult;
	"#
                )?;
        }

        writeln!(
            writer,
            r#"
}}"#,
        )?;
    }
    write!(
        writer,
        r#"
}}"#
    )?;

    Ok(())
}
