@page
@using Microsoft.AspNetCore.Authorization
@using Volo.Abp.AspNetCore.Mvc.UI.Layout
@using ${domain_name}.Permissions
@using ${domain_name}.Web.Pages.${entity_plural}
@using ${domain_name}.Web.Menus
@using Microsoft.AspNetCore.Mvc.Localization
@using ${domain_name}.Localization
@inject IHtmlLocalizer<${domain_short}Resource> L
@inject IAuthorizationService Authorization
@model IndexModel
@inject IPageLayout PageLayout
@{
  PageLayout.Content.Title = L["${entity_plural}"].Value;
  PageLayout.Content.MenuItemName = ${domain_short}Menus.${entity_plural};
}

@section styles { <abp-style src="/Pages/${entity_plural}/Index.css" /> }
@section scripts { <abp-script src="/Pages/${entity_plural}/index.js" /> }

@section content_toolbar {
  <abp-button id="ExportToExcelButton" text="@L["ExportToExcel"].Value" icon="download" size="Small" button-type="Success" class="me-2"/>
  @if (await Authorization.IsGrantedAsync(${domain_short}Permissions.${entity_plural}.Create)) {
    <abp-button id="New${entity_name}Button" text="@L["New${entity_name}"].Value" icon="plus" size="Small" button-type="Primary" />
  }
}

<abp-card>
  <abp-card-body>
    <abp-row class="mb-3">
      <div class="p-2">
        <form id="SearchForm" autocomplete="off">
          <div class="d-flex flex-row align-items-center">
            <div class="flex-fill me-2">
              <input class="form-control page-search-filter-text" id="FilterText" placeholder="@L["Search"]" />
            </div>
            <div class="btn-group" role="group">
              <abp-button button-type="Outline_Primary" type="submit" icon="search" />
              <abp-button button-type="Outline_Success" id="AdvancedFilterSectionToggler" type="button" icon="filter" />
              <abp-button button-type="Outline_Danger" type="button" id="ClearFiltersButton" icon="times" />
            </div>
          </div>
        </form>
      </div>
    </abp-row>

    <abp-row id="AdvancedFilterSection" class="mt-3" style="display:none;">
      ${web_index_filter_fields}
    </abp-row>

    <div id="bulk-delete-context-menu" class="d-none">
      <div class="d-flex justify-content-between align-items-center mb-2">
        <p class="lead mb-0 d-none" id="items-selected-info-message"></p>
        <div>
          <button class="btn btn-outline-secondary d-none mx-1" id="select-all-items-btn"></button>
          <button class="btn btn-outline-secondary d-none mx-1" id="clear-selection-btn">@L["ClearSelection"]</button>
          <button class="btn btn-danger mx-1" id="delete-selected-items"><i class="fa fa-trash"></i> @L["Delete"]</button>
        </div>
      </div>
      <hr class="my-1 mx-0" />
    </div>

    <abp-table striped-rows="true" id="${entity_plural}Table">
      <thead>
        <tr>
          <th id="BulkDeleteCheckboxTheader"><input type="checkbox" id="select_all" class="form-check-input" /></th>
          <th>@L["Actions"]</th>
          ${web_index_column_fields}
        </tr>
      </thead>
    </abp-table>
  </abp-card-body>
</abp-card>
