using Microsoft.AspNetCore.Mvc;
using Volo.Abp.AspNetCore.Mvc;
using Volo.Abp.AspNetCore.Mvc.UI.Widgets;

namespace ${domain_name}.Web.Components.ChangeTheme;

[Widget(ScriptFiles = new[] { "/Components/ChangeTheme/ChangeTheme.js" })]
public class ChangeThemeViewComponent : AbpViewComponent
{
  public IViewComponentResult Invoke()
  {
    return View("~/Components/ChangeTheme/Default.cshtml");
  }
}
