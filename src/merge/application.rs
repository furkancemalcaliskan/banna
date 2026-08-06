use super::{MergeContext, ensure_using, insert_before_last_close_brace_of_method};
use crate::utils::{read_text, write_text};
use anyhow::Result;
use regex::Regex;
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::Path;

pub(super) fn merge_application(
    out_root: &Path,
    context: &MergeContext<'_>,
    mapping: &HashMap<String, String>,
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
    let namespace = context.namespace.to_owned();

    let app_dir = out_root.join(format!("{}.Application", domain));
    let profile_file = app_dir.join(format!("{}ApplicationAutoMapperProfile.cs", short));
    let mut desired: Vec<(Regex, String)> = vec![
        (
            Regex::new(&format!(
                r"CreateMap\s*<\s*{}\s*,\s*LookupDto\s*<\s*Guid\s*>\s*>\s*\(",
                regex::escape(&entity)
            ))?,
            format!(
                "CreateMap<{e}, LookupDto<Guid>>()    .ForMember(dest => dest.DisplayName, opt => opt.MapFrom(src => src.Name));",
                e = entity
            ),
        ),
        (
            Regex::new(&format!(
                r"CreateMap\s*<\s*{}\s*,\s*{}\s*Dto\s*>\s*\(",
                regex::escape(&entity),
                regex::escape(&entity)
            ))?,
            format!("CreateMap<{e}, {e}Dto>();", e = entity),
        ),
        (
            Regex::new(&format!(
                r"CreateMap\s*<\s*{}\s*,\s*{}\s*ExcelDto\s*>\s*\(",
                regex::escape(&entity),
                regex::escape(&entity)
            ))?,
            format!("CreateMap<{e}, {e}ExcelDto>();", e = entity),
        ),
        (
            Regex::new(&format!(
                r"CreateMap\s*<\s*{}WithNavigationProperties\s*,\s*{}WithNavigationPropertiesDto\s*>\s*\(",
                regex::escape(&entity),
                regex::escape(&entity)
            ))?,
            format!(
                "CreateMap<{e}WithNavigationProperties, {e}WithNavigationPropertiesDto>();",
                e = entity
            ),
        ),
    ];

    desired.push((
        Regex::new(&format!(
            r"CreateMap\s*<\s*{}\s*,\s*{}\s*CreateDto\s*>\s*\(",
            regex::escape(&entity),
            regex::escape(&entity)
        ))?,
        format!("CreateMap<{e}, {e}CreateDto>();", e = entity),
    ));

    desired.push((
        Regex::new(&format!(
            r"CreateMap\s*<\s*{}\s*,\s*{}\s*UpdateDto\s*>\s*\(",
            regex::escape(&entity),
            regex::escape(&entity)
        ))?,
        format!("CreateMap<{e}, {e}UpdateDto>();", e = entity),
    ));

    if let Some(nav_maps) = mapping.get("nav_lookup_profile_maps") {
        for line in nav_maps.lines().map(str::trim).filter(|l| !l.is_empty()) {
            let presence = Regex::new(&regex::escape(line).replace(r"\ ", r"\s+").to_string())?;
            desired.push((presence, line.to_string()));
        }
    }
    let app_mapper_file = app_dir.join(format!("{}ApplicationMappers.cs", short));
    if profile_file.exists() {
        let mut src = read_text(&profile_file)?;

        if !src.contains("using System;") {
            if let Some(first_using_pos) = src.find("using ") {
                src.insert_str(first_using_pos, "using System;\n");
            } else {
                src = format!("using System;\n{}", src);
            }
            write_text(&profile_file, &src)?;
            report!("merged: {} (+using System;)", profile_file.display());
        }

        let want_entity_using = format!("using {};", namespace);
        let want_shared_using = format!("using {}.Shared;", domain);
        let s1 = ensure_using(&src, &want_entity_using);
        let mut s2 = ensure_using(&s1, &want_shared_using);

        if let Some(nav_us) = mapping.get("nav_usings") {
            for u in nav_us.lines().filter(|l| !l.trim().is_empty()) {
                let next = ensure_using(&s2, u);
                if next != s2 {
                    s2 = next;
                }
            }
        }

        if s2 != src {
            src = s2;
            write_text(&profile_file, &src)?;
            report!(
                "merged: {} (+usings for entity, .Shared and nav usings)",
                profile_file.display()
            );
        }

        let ctor_rx = Regex::new(&format!(
            r"public\s+{}\s*ApplicationAutoMapperProfile\s*\(\s*\)\s*",
            regex::escape(&short)
        ))?;

        let mut changed = false;
        for (presence_rx, line) in &desired {
            if !presence_rx.is_match(&src) {
                let to_insert = if line.ends_with(';') {
                    line.clone()
                } else {
                    format!("{line};")
                };
                let new_src = insert_before_last_close_brace_of_method(
                    &src,
                    &ctor_rx,
                    &format!("\n        {}", to_insert),
                );
                if new_src != src {
                    src = new_src;
                    changed = true;
                    report!(
                        "merged: {} (+{})",
                        profile_file.display(),
                        line.split('(').next().unwrap_or("CreateMap")
                    );
                } else {
                    report!("warn: ctor bulunamadı: {}", profile_file.display());
                }
            }
        }
        if changed {
            write_text(&profile_file, &src)?;
        }
    } else {
        report!("warn: profile not found: {}", profile_file.display());

        if app_mapper_file.exists() {
            let mut src = read_text(&app_mapper_file)?;
            let want_usings = [
                format!("using {};", namespace),
                format!("using {}.Shared;", domain),
                "using Riok.Mapperly.Abstractions;".to_string(),
                "using Volo.Abp.Mapperly;".to_string(),
                "using System;".to_string(),
            ];
            let mut s = src.clone();
            for u in &want_usings {
                s = ensure_using(&s, u);
            }
            if let Some(nav_us) = mapping.get("nav_usings") {
                for u in nav_us.lines().filter(|l| !l.trim().is_empty()) {
                    s = ensure_using(&s, u);
                }
            }

            let mut changed = s != src;
            src = s;

            let class_mark = format!("class {}To{}DtoMapper", entity, entity);
            let has_lookup_mapper = src.contains(&format!("class {}ToLookupDtoMapper", entity));
            if !src.contains(&class_mark) {
                let lookup_block = if has_lookup_mapper {
                    String::new()
                } else {
                    format!(
                        r#"
    [Mapper]
    public partial class {e}ToLookupDtoMapper : MapperBase<{e}, LookupDto<Guid>>
    {{
    [MapProperty(nameof({e}.{display}), nameof(LookupDto<Guid>.DisplayName))]
    public override partial LookupDto<Guid> Map({e} source);
    [MapProperty(nameof({e}.{display}), nameof(LookupDto<Guid>.DisplayName))]
    public override partial void Map({e} source, LookupDto<Guid> destination);
    }}
    "#,
                        e = entity,
                        display = mapping
                            .get("entity_lookup_display_prop")
                            .map(|s| s.as_str())
                            .unwrap_or("Name")
                    )
                };

                let mapper_block = format!(
                    r#"
    [Mapper]
    public partial class {e}To{e}DtoMapper : MapperBase<{e}, {e}Dto>
    {{
    public override partial {e}Dto Map({e} source);
    public override partial void Map({e} source, {e}Dto destination);
    }}
    
    [Mapper]
    public partial class {e}To{e}ExcelDtoMapper : MapperBase<{e}, {e}ExcelDto>
    {{
    public override partial {e}ExcelDto Map({e} source);
    public override partial void Map({e} source, {e}ExcelDto destination);
    }}
    
    [Mapper]
    public partial class {e}WithNavigationPropertiesTo{e}WithNavigationPropertiesDtoMapper : MapperBase<{e}WithNavigationProperties, {e}WithNavigationPropertiesDto>
    {{
    public override partial {e}WithNavigationPropertiesDto Map({e}WithNavigationProperties source);
    public override partial void Map({e}WithNavigationProperties source, {e}WithNavigationPropertiesDto destination);
    }}
    
    {lookup}"#,
                    e = entity,
                    lookup = lookup_block
                );

                src.push_str(&mapper_block);
                changed = true;
                report!(
                    "merged: {} (+Mapperly classes for {})",
                    app_mapper_file.display(),
                    entity
                );
            } else {
                report!(
                    "skip: Mapperly classes for {} already present in {}",
                    entity,
                    app_mapper_file.display()
                );
            }

            if let Some(nav_blocks) = mapping.get("nav_lookup_mapperly_blocks") {
                let nav_rx = Regex::new(
                    r"(?s)// <auto-nav-lookup-([A-Za-z_][A-Za-z0-9_]*)>.*?// </auto-nav-lookup-[A-Za-z_][A-Za-z0-9_]*>",
                )?;
                for cap in nav_rx.captures_iter(nav_blocks) {
                    let marker = cap.get(1).map(|m| m.as_str()).unwrap_or_default();
                    let block = cap.get(0).map(|m| m.as_str()).unwrap_or_default();
                    let class_marker = format!("class {}ToLookupDtoMapper", marker);
                    if !marker.is_empty()
                        && !block.is_empty()
                        && !src.contains(&format!("<auto-nav-lookup-{}>", marker))
                        && !src.contains(&class_marker)
                    {
                        src.push('\n');
                        src.push_str(block.trim());
                        src.push('\n');
                        changed = true;
                        report!(
                            "merged: {} (+Mapperly lookup mapper for {})",
                            app_mapper_file.display(),
                            marker
                        );
                    }
                }
            }

            if changed {
                write_text(&app_mapper_file, &src)?;
            }
        } else {
            report!(
                "warn: profile missing and no fallback mapper file found ({})",
                app_mapper_file.display()
            );
        }
    }

    Ok(())
}
