@page
@using Microsoft.AspNetCore.Mvc.Localization
@using ${domain_name}.Localization
@using Volo.Abp.AspNetCore.Mvc.UI.Bootstrap.TagHelpers.Modal
@using ${namespace}
@using ${web_namespace}
@inject IHtmlLocalizer<${domain_short}Resource> L
@model DetailModalModel
@{ Layout = null; }

@section styles { <abp-style src="/Pages/${entity_plural}/DetailModal.css" /> }

<form data-ajaxForm="true" asp-page="/${entity_plural}/DetailModal" autocomplete="off">
  <abp-modal id="${entity_name}DetailModal" size="Large" centered="true">
    <abp-modal-header title="@L["Detail"].Value" class="border-1 border-bottom mb-3"></abp-modal-header>
    <abp-modal-body>
        <input asp-for="Id" hidden="true" />
        <input asp-for="${entity_name}.ConcurrencyStamp" hidden="true" />
      ${web_form_fields_detail}
    </abp-modal-body>
    <abp-modal-footer buttons="@(AbpModalButtons.Close)">
    </abp-modal-footer>
  </abp-modal>
</form>
