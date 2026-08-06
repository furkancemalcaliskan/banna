use super::{
    MergeContext, ensure_using, insert_before_last_close_brace_of_method, insert_inside_class,
};
use crate::templates::read_tpl_text;
use crate::utils::{read_text, write_text};
use anyhow::Result;
use regex::Regex;
use std::cell::RefCell;
use std::fs;
use std::path::Path;

fn remove_mapperly_class(source: &str, class_name: &str) -> Result<(String, bool)> {
    let declaration = Regex::new(&format!(
        r"(?m)^[ \t]*\[Mapper(?:\([^\r\n]*\))?\][ \t]*\r?\n[ \t]*public[ \t]+partial[ \t]+class[ \t]+{}\b",
        regex::escape(class_name)
    ))?;
    let Some(found) = declaration.find(source) else {
        return Ok((source.to_owned(), false));
    };
    let Some(open_offset) = source[found.end()..].find('{') else {
        return Ok((source.to_owned(), false));
    };

    let open = found.end() + open_offset;
    let mut depth = 0_u32;
    let mut close = None;
    for (offset, byte) in source.as_bytes()[open..].iter().enumerate() {
        match byte {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    close = Some(open + offset + 1);
                    break;
                }
            }
            _ => {}
        }
    }
    let Some(mut end) = close else {
        return Ok((source.to_owned(), false));
    };
    while source
        .as_bytes()
        .get(end)
        .is_some_and(u8::is_ascii_whitespace)
    {
        end += 1;
    }

    let mut cleaned = String::with_capacity(source.len() - (end - found.start()));
    cleaned.push_str(&source[..found.start()]);
    cleaned.push_str(&source[end..]);
    Ok((cleaned, true))
}

fn cleanup_razor_viewmodel_mappings(source: &str, entity: &str) -> Result<(String, usize)> {
    let auto_mapper_line = Regex::new(&format!(
        r"(?m)^[ \t]*CreateMap<{}Dto,[ \t]*{}(?:Update|Detail)ViewModel>\(\);[ \t]*\r?\n?",
        regex::escape(entity),
        regex::escape(entity)
    ))?;
    let mut cleaned = auto_mapper_line.replace_all(source, "").into_owned();
    let mut removed = usize::from(cleaned != source);

    for class_name in [
        format!("{entity}DtoTo{entity}UpdateViewModelMapper"),
        format!("{entity}UpdateViewModelTo{entity}UpdateDtoMapper"),
        format!("{entity}CreateViewModelTo{entity}CreateDtoMapper"),
        format!("{entity}DtoTo{entity}DetailViewModelMapper"),
    ] {
        let (next, did_remove) = remove_mapperly_class(&cleaned, &class_name)?;
        cleaned = next;
        removed += usize::from(did_remove);
    }

    Ok((cleaned, removed))
}

pub(super) fn merge_web(
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
    let web_namespace = context.web_namespace.to_owned();
    let is_mvc = context.is_mvc;
    let is_razor = context.is_razor;

    if is_mvc {
        let web_menus_dir = out_root.join(format!("{}.Web", domain)).join("Menus");
        let menus_cs_file = web_menus_dir.join(format!("{}Menus.cs", short));
        if menus_cs_file.exists() {
            let mut src = read_text(&menus_cs_file)?;

            let new_const_line = format!(
                "\n    public const string {} = Prefix + \".{}\";\n",
                entity_plural, entity_plural
            );

            let already = src.contains(&format!("public const string {} = ", entity_plural));
            if !already {
                let rx = Regex::new(&format!(
                    r"public\s+class\s+{}\s*Menus\s*",
                    regex::escape(&short)
                ))?;

                let new_src = insert_inside_class(&src, &rx, &new_const_line, true);
                if new_src != src {
                    src = new_src;
                    write_text(&menus_cs_file, &src)?;
                    report!(
                        "merged: {} (+const {} = Prefix + \".{}\")",
                        menus_cs_file.display(),
                        entity_plural,
                        entity_plural
                    );
                } else {
                    report!(
                        "warn: could not insert menu const into {}",
                        menus_cs_file.display()
                    );
                }
            } else {
                report!(
                    "skip: menu const for {} already present in {}",
                    entity_plural,
                    menus_cs_file.display()
                );
            }
        } else {
            report!("warn: menus file not found: {}", menus_cs_file.display());
        }

        let contributor_file = web_menus_dir.join(format!("{}MenuContributor.cs", short));
        if contributor_file.exists() {
            let mut src = read_text(&contributor_file)?;

            let want_perm_using = format!("using {}.Permissions;", domain);
            let s2 = ensure_using(&src, &want_perm_using);
            let mut changed = false;
            if s2 != src {
                src = s2;
                changed = true;
            }

            let marker = format!("{}Menus.{}", short, entity_plural);
            let already = src.contains(&marker);

            if !already {
                let add_block = format!(
                    r#"
        context.Menu.AddItem(
            new ApplicationMenuItem(
                {short}Menus.{ep},
                l["Menu:{ep}"],
                url: "/{ep}",
                icon: "fa fa-file-alt",
                requiredPermissionName: {short}Permissions.{ep}.Default)
        );
    "#,
                    short = short,
                    ep = entity_plural
                );

                let needle = "return Task.CompletedTask;";
                if let Some(pos) = src.find(needle) {
                    let mut new_src = String::with_capacity(src.len() + add_block.len() + 4);
                    new_src.push_str(&src[..pos]);
                    new_src.push_str(&add_block);
                    new_src.push_str(&src[pos..]);
                    src = new_src;
                    changed = true;
                    report!(
                        "merged: {} (+menu AddItem for {})",
                        contributor_file.display(),
                        entity_plural
                    );
                } else {
                    let rx = Regex::new(
                        r"private\s+static\s+Task\s+ConfigureMainMenuAsync\s*\(\s*MenuConfigurationContext\s+context\s*\)\s*",
                    )?;
                    let new_src = insert_before_last_close_brace_of_method(&src, &rx, &add_block);
                    if new_src != src {
                        src = new_src;
                        changed = true;
                        report!(
                            "merged: {} (+menu AddItem for {} @method-end)",
                            contributor_file.display(),
                            entity_plural
                        );
                    } else {
                        report!(
                            "warn: could not locate ConfigureMainMenuAsync() in {}; no menu entry inserted.",
                            contributor_file.display()
                        );
                    }
                }
            } else {
                report!(
                    "skip: menu AddItem for {} already present in {}",
                    entity_plural,
                    contributor_file.display()
                );
            }

            if changed {
                write_text(&contributor_file, &src)?;
            }
        } else {
            report!(
                "warn: menu contributor not found: {}",
                contributor_file.display()
            );
        }

        if is_razor {
            let web_profile_candidates = [
                out_root
                    .join(format!("{}.Web", domain))
                    .join("AutoMapper")
                    .join(format!("{}WebAutoMapperProfile.cs", short)),
                out_root
                    .join(format!("{}.Web", domain))
                    .join(format!("{}WebAutoMapperProfile.cs", short)),
            ];

            let maps = format!(
                "\n        CreateMap<{e}Dto, {e}UpdateViewModel>();\n        CreateMap<{e}Dto, {e}DetailViewModel>();\n",
                e = entity
            );

            let patch_web_profile = |web_profile_file: &Path| -> Result<bool> {
                if !web_profile_file.exists() {
                    report!(
                        "warn: Web AutoMapper profile not found: {}",
                        web_profile_file.display()
                    );
                    return Ok(false);
                }

                let mut src = read_text(web_profile_file)?;

                let want_using_entity_ns = format!("using {};", namespace);
                let want_using_web_pages = format!("using {};", web_namespace);

                let s1 = ensure_using(&src, &want_using_entity_ns);
                let s2 = ensure_using(&s1, &want_using_web_pages);
                if s2 != src {
                    src = s2;
                    write_text(web_profile_file, &src)?;
                    report!(
                        "merged: {} (+usings for Contracts & Web.Pages)",
                        web_profile_file.display()
                    );
                }

                let already = src.contains(&format!(
                    "CreateMap<{}Dto, {}UpdateViewModel>",
                    entity, entity
                )) || src.contains(&format!(
                    "CreateMap<{}Dto, {}DetailViewModel>",
                    entity, entity
                ));

                if already {
                    report!(
                        "skip: Web AutoMapper maps for {} already present in {}",
                        entity,
                        web_profile_file.display()
                    );
                    return Ok(true);
                }

                let ctor_rx = Regex::new(&format!(
                    r"public\s+{}\s*WebAutoMapperProfile\s*\(\s*\)\s*",
                    regex::escape(&short)
                ))?;
                let mut new_src = insert_before_last_close_brace_of_method(&src, &ctor_rx, &maps);
                if new_src != src {
                    write_text(web_profile_file, &new_src)?;
                    report!(
                        "merged: {} (+Web AutoMapper maps for {} in ctor)",
                        web_profile_file.display(),
                        entity
                    );
                    return Ok(true);
                }

                let class_rx = Regex::new(&format!(
                    r"class\s+{}\s*WebAutoMapperProfile\s*:\s*Profile",
                    regex::escape(&short)
                ))?;
                new_src = insert_inside_class(&src, &class_rx, &maps, true);
                if new_src != src {
                    write_text(web_profile_file, &new_src)?;
                    report!(
                        "merged: {} (+Web AutoMapper maps for {} @class-end)",
                        web_profile_file.display(),
                        entity
                    );
                    return Ok(true);
                }

                src.push_str(&maps);
                write_text(web_profile_file, &src)?;
                report!(
                    "merged: {} (+Web AutoMapper maps for {} @append)",
                    web_profile_file.display(),
                    entity
                );
                Ok(true)
            };

            let patch_web_mappers = |web_mapper_file: &Path| -> Result<bool> {
                if !web_mapper_file.exists() {
                    report!(
                        "warn: Web mapper fallback not found: {}",
                        web_mapper_file.display()
                    );
                    return Ok(false);
                }

                let mut src = read_text(web_mapper_file)?;

                let want_using_entity_ns = format!("using {};", namespace);
                let want_using_web_pages = format!("using {};", web_namespace);
                let mut s = src.clone();
                for u in [
                    &want_using_entity_ns,
                    &want_using_web_pages,
                    "using Volo.Abp.Mapperly;",
                ] {
                    s = ensure_using(&s, u);
                }
                s = ensure_using(&s, "using Riok.Mapperly.Abstractions;");

                let class_mark =
                    format!("class {e}UpdateViewModelTo{e}UpdateDtoMapper", e = entity);
                let already = s.contains(&class_mark)
                    || s.contains(&format!(
                        "class {e}DtoTo{e}UpdateViewModelMapper",
                        e = entity
                    ))
                    || s.contains(&format!(
                        "class {e}CreateViewModelTo{e}CreateDtoMapper",
                        e = entity
                    ))
                    || s.contains(&format!(
                        "class {e}DtoTo{e}DetailViewModelMapper",
                        e = entity
                    ));
                let mut changed = s != src;
                src = s;

                if !already {
                    let mapper_block = format!(
                        r#"
    [Mapper]
    public partial class {e}DtoTo{e}UpdateViewModelMapper : MapperBase<{e}Dto, {e}UpdateViewModel>
    {{
    public override partial {e}UpdateViewModel Map({e}Dto source);
    public override partial void Map({e}Dto source, {e}UpdateViewModel destination);
    }}
    
    [Mapper]
    public partial class {e}UpdateViewModelTo{e}UpdateDtoMapper : MapperBase<{e}UpdateViewModel, {e}UpdateDto>
    {{
    public override partial {e}UpdateDto Map({e}UpdateViewModel source);
    public override partial void Map({e}UpdateViewModel source, {e}UpdateDto destination);
    }}
    
    [Mapper]
    public partial class {e}CreateViewModelTo{e}CreateDtoMapper : MapperBase<{e}CreateViewModel, {e}CreateDto>
    {{
    public override partial {e}CreateDto Map({e}CreateViewModel source);
    public override partial void Map({e}CreateViewModel source, {e}CreateDto destination);
    }}
    
    [Mapper]
    public partial class {e}DtoTo{e}DetailViewModelMapper : MapperBase<{e}Dto, {e}DetailViewModel>
    {{
    public override partial {e}DetailViewModel Map({e}Dto source);
    public override partial void Map({e}Dto source, {e}DetailViewModel destination);
    }}
    "#,
                        e = entity
                    );

                    src.push_str(&mapper_block);
                    changed = true;
                    report!(
                        "merged: {} (+Web Mapperly classes for {})",
                        web_mapper_file.display(),
                        entity
                    );
                } else {
                    report!(
                        "skip: Web maps for {} already present in {}",
                        entity,
                        web_mapper_file.display()
                    );
                }

                if changed {
                    write_text(web_mapper_file, &src)?;
                }
                Ok(changed)
            };

            let mut done = false;
            for p in &web_profile_candidates {
                if patch_web_profile(p)? {
                    done = true;
                    break;
                }
            }
            if !done {
                let web_mapper_candidates = [
                    out_root
                        .join(format!("{}.Web", domain))
                        .join("AutoMapper")
                        .join(format!("{}WebMappers.cs", short)),
                    out_root
                        .join(format!("{}.Web", domain))
                        .join(format!("{}WebMappers.cs", short)),
                ];
                for p in &web_mapper_candidates {
                    if patch_web_mappers(p)? {
                        done = true;
                        break;
                    }
                }
                if !done {
                    report!("warn: no Web AutoMapper profile patched (no candidate matched).");
                }
            }
        } else {
            let web_mapping_candidates = [
                out_root
                    .join(format!("{}.Web", domain))
                    .join("AutoMapper")
                    .join(format!("{}WebAutoMapperProfile.cs", short)),
                out_root
                    .join(format!("{}.Web", domain))
                    .join(format!("{}WebAutoMapperProfile.cs", short)),
                out_root
                    .join(format!("{}.Web", domain))
                    .join("AutoMapper")
                    .join(format!("{}WebMappers.cs", short)),
                out_root
                    .join(format!("{}.Web", domain))
                    .join(format!("{}WebMappers.cs", short)),
            ];

            for path in web_mapping_candidates.iter().filter(|path| path.exists()) {
                let source = read_text(path)?;
                let (cleaned, removed) = cleanup_razor_viewmodel_mappings(&source, &entity)?;
                if removed > 0 {
                    write_text(path, &cleaned)?;
                    report!(
                        "cleaned: {} (-{} obsolete Razor ViewModel mapping block(s) for {})",
                        path.display(),
                        removed,
                        entity
                    );
                }
            }
        }

        let tpl_rel = "Web/MVC/wwwroot/global-styles.css";
        match read_tpl_text(tpl_rel) {
            Ok(tpl) => {
                if tpl.trim().is_empty() {
                    report!("warn: embedded global-styles.css is empty, skipping.");
                } else {
                    let target = out_root
                        .join(format!("{}.Web", domain))
                        .join("wwwroot")
                        .join("global-styles.css");

                    if let Some(parent) = target.parent() {
                        let _ = fs::create_dir_all(parent);
                    }

                    let mut should_write = true;
                    if target.exists()
                        && let Ok(existing) = fs::read_to_string(&target)
                        && existing.contains(".select2")
                    {
                        report!(
                            "skip: global-styles.css already contains select2 styles -> {}",
                            target.display()
                        );
                        should_write = false;
                    }

                    if should_write {
                        if let Err(e) = fs::write(&target, tpl) {
                            report!(
                                "warn: failed to write global-styles.css ({}): {}",
                                target.display(),
                                e
                            );
                        } else {
                            report!(
                                "merged: {} (+global-styles.css from embedded asset)",
                                target.display()
                            );
                        }
                    }
                }
            }
            Err(e) => {
                report!(
                    "warn: global-styles.css not found in embedded templates: {}",
                    e
                );
            }
        }
    }

    Ok(())
}
