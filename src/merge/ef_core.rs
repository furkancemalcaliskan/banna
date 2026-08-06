use super::{
    MergeContext, ensure_using, insert_before_last_close_brace_of_method, insert_inside_class,
};
use crate::utils::{read_text, write_text};
use anyhow::Result;
use regex::Regex;
use std::cell::RefCell;
use std::path::Path;

pub(super) fn merge_ef_core(
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
    let entity = context.entity.to_owned();
    let entity_plural = context.entity_plural.to_owned();
    let namespace = context.namespace.to_owned();

    let ef_dir = out_root
        .join(format!("{}.EntityFrameworkCore", domain))
        .join("EntityFrameworkCore");
    let dbctx_file = ef_dir.join(format!("{}DbContext.cs", short));
    if dbctx_file.exists() {
        let mut src = read_text(&dbctx_file)?;
        src = ensure_using(&src, &format!("using {};", namespace));
        let dbset_line = format!(
            "public DbSet<{e}> {ep} {{ get; set; }} = null!;",
            e = entity,
            ep = entity_plural
        );
        if !src.contains(&dbset_line) {
            let rx = Regex::new(&format!(
                r"class\s+{}\s*DbContext\s*:",
                regex::escape(&short)
            ))?;
            src = insert_inside_class(
                &src,
                &rx,
                &format!("\n    {dbset}\n", dbset = dbset_line),
                false,
            );
            report!("merged: {} (+DbSet)", dbctx_file.display());
        } else {
            report!("skip: DbSet already present in {}", dbctx_file.display());
        }
        let call = format!("builder.Configure{}();", entity);
        if !src.contains(&call) {
            let rx = Regex::new(
                r"protected\s+override\s+void\s+OnModelCreating\s*\(\s*ModelBuilder\s+builder\s*\)\s*",
            )?;
            src =
                insert_before_last_close_brace_of_method(&src, &rx, &format!("\n        {call}\n"));
            report!("merged: {} (+OnModelCreating call)", dbctx_file.display());
        } else {
            report!(
                "skip: OnModelCreating call already present in {}",
                dbctx_file.display()
            );
        }
        write_text(&dbctx_file, &src)?;
    } else {
        report!("warn: DbContext not found: {}", dbctx_file.display());
    }

    Ok(())
}
