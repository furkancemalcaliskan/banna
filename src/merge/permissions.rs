use super::{MergeContext, insert_before_last_close_brace_of_method, insert_inside_class};
use crate::utils::{read_text, write_text};
use anyhow::Result;
use regex::Regex;
use std::cell::RefCell;
use std::path::Path;

pub(super) fn merge_permissions(
    out_root: &Path,
    context: &MergeContext<'_>,
    reporter: &RefCell<&mut dyn FnMut(&str)>,
) -> Result<()> {
    macro_rules! report {
        ($($arg:tt)*) => {
            (reporter.borrow_mut())(&format!($($arg)*))
        };
    }
    let domain = context.domain.to_owned();
    let short = context.short.to_owned();
    let entity_plural = context.entity_plural.to_owned();
    let entity_lower = context.entity_lower.to_owned();

    let contracts_dir = out_root.join(format!("{}.Application.Contracts", domain));
    let perms_dir = contracts_dir.join("Permissions");
    let perms_file = perms_dir.join(format!("{}Permissions.cs", short));
    if perms_file.exists() {
        let mut src = read_text(&perms_file)?;
        let nested = format!(
            "\n    public static class {ep}\n    {{\n        public const string Default = GroupName + \".{ep}\";\n         public const string Detail = Default + \".Detail\";\n       public const string Edit = Default + \".Edit\";\n        public const string Create = Default + \".Create\";\n        public const string Delete = Default + \".Delete\";\n    }}\n",
            ep = entity_plural
        );
        if !src.contains(&format!("class {}", entity_plural)) {
            let rx = Regex::new(&format!(
                r"public\s+static\s+class\s+{}Permissions",
                regex::escape(&short)
            ))?;
            let new_src = insert_inside_class(&src, &rx, &nested, true);
            if new_src != src {
                src = new_src;
                write_text(&perms_file, &src)?;
                report!(
                    "merged: {} (+{} permissions)",
                    perms_file.display(),
                    entity_plural
                );
            } else {
                report!(
                    "warn: could not insert permissions into {}",
                    perms_file.display()
                );
            }
        } else {
            report!(
                "skip: permissions for {} already present in {}",
                entity_plural,
                perms_file.display()
            );
        }
    } else {
        report!("warn: permissions file not found: {}", perms_file.display());
    }

    let pdp_file = perms_dir.join(format!("{}PermissionDefinitionProvider.cs", short));
    if pdp_file.exists() {
        let mut src = read_text(&pdp_file)?;
        let add = format!(
            "\n        var {el}Permission = myGroup.AddPermission({short}Permissions.{ep}.Default, L(\"Permission:{ep}\"));\n        {el}Permission.AddChild({short}Permissions.{ep}.Create, L(\"Permission:Create\"));\n         {el}Permission.AddChild({short}Permissions.{ep}.Detail, L(\"Permission:Detail\"));\n       {el}Permission.AddChild({short}Permissions.{ep}.Edit, L(\"Permission:Edit\"));\n        {el}Permission.AddChild({short}Permissions.{ep}.Delete, L(\"Permission:Delete\"));\n",
            el = entity_lower,
            ep = entity_plural,
            short = short
        );
        if !src.contains(&format!("var {}Permission", entity_lower)) {
            let rx = Regex::new(
                r"public\s+override\s+void\s+Define\s*\(\s*IPermissionDefinitionContext\s+context\s*\)\s*",
            )?;
            let new_src = insert_before_last_close_brace_of_method(&src, &rx, &add);
            if new_src != src {
                src = new_src;
                write_text(&pdp_file, &src)?;
                report!("merged: {} (+Define() permissions)", pdp_file.display());
            } else {
                report!(
                    "warn: could not insert into Define() in {}",
                    pdp_file.display()
                );
            }
        } else {
            report!(
                "skip: Define() lines already present in {}",
                pdp_file.display()
            );
        }
    } else {
        report!(
            "warn: permission provider not found: {}",
            pdp_file.display()
        );
    }

    Ok(())
}
