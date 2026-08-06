using ${domain_name}.Web.Components.ChangeTheme;
using System.Threading.Tasks;
using Volo.Abp.AspNetCore.Mvc.UI.Theme.Shared.Toolbars;

namespace ${domain_name}.Web.Toolbars;

public class ChangeThemeToolbarContributor : IToolbarContributor
{
    public Task ConfigureToolbarAsync(IToolbarConfigurationContext context)
    {
        if (context.Toolbar.Name == StandardToolbars.Main)
        {
            context.Toolbar.Items
                .Add(new ToolbarItem(typeof(ChangeThemeViewComponent)));
        }

        return Task.CompletedTask;
    }
}