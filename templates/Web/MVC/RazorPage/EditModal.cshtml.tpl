@page
@using Microsoft.AspNetCore.Mvc.Localization
@using ${domain_name}.Localization
@using Volo.Abp.AspNetCore.Mvc.UI.Bootstrap.TagHelpers.Modal
@using ${namespace}
@using ${web_namespace}
@inject IHtmlLocalizer<${domain_short}Resource> L
@model EditModalModel
@{ Layout = null; }

@section styles { <abp-style src="/Pages/${entity_plural}/EditModal.css" /> }

<form data-ajaxForm="true" asp-page="/${entity_plural}/EditModal" autocomplete="off">
  <abp-modal id="${entity_name}EditModal" size="Large" centered="true">
    <abp-modal-header title="@L["Edit"].Value" class="border-1 border-bottom mb-3"></abp-modal-header>
    <abp-modal-body>
        <input asp-for="Id" hidden="true" />
        <input asp-for="${entity_name}.ConcurrencyStamp" hidden="true" />
      ${web_form_fields_edit}
    </abp-modal-body>
    <abp-modal-footer buttons="@(AbpModalButtons.Cancel|AbpModalButtons.Save)">
    </abp-modal-footer>
  </abp-modal>
</form>
