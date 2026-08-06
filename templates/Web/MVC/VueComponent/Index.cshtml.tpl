@page
@using Microsoft.AspNetCore.Authorization
@using Volo.Abp.AspNetCore.Mvc.UI.Layout
@using ${domain_name}.Permissions
@using ${web_namespace}
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
  <abp-button id="ExportToExcelButton" text="@L["ExportToExcel"].Value" icon="download" size="Small" button-type="Success" class="me-2" />
  @if (await Authorization.IsGrantedAsync(${domain_short}Permissions.${entity_plural}.Create))
  {
    <abp-button id="New${entity_name}Button" text="@L["New${entity_name}"].Value" icon="plus" size="Small" button-type="Primary" />
  }
}

<script type="text/x-template" id="${entity_name_lower}-list-template">
  <div class="card mb-3">
    <div class="card-body">
      <div class="row mb-3">
        <div class="p-2">
          <div class="d-flex flex-row align-items-center">
            <div class="flex-fill me-2">
              <input id="FilterText"
                     class="form-control page-search-filter-text"
                     v-model="filter.filterText"
                     placeholder="@L["Search"].Value" />
            </div>
            <div class="btn-group" role="group">
              <button type="button" class="btn btn-outline-primary btn-icon" title="Search" @@click="onGetFilteredData">
                <i class="fa fa-search"></i>
              </button>
              <button type="button" id="AdvancedFilterSectionToggler" class="btn btn-outline-success btn-icon" title="Filter" @@click="toggleAdvancedFilter">
                <i class="fa fa-filter"></i>
              </button>
              <button type="button" id="ClearFiltersButton" class="btn btn-outline-danger btn-icon" title="Clear" @@click="clearFilters">
                <i class="fa fa-times"></i>
              </button>
            </div>
          </div>
        </div>
      </div>

      <div v-show="showAdvancedFilter" class="row mt-3">
        ${vue_index_advanced_filter_inputs}
      </div>

      <div id="bulk-delete-context-menu" class="d-none">
        <div class="d-flex justify-content-between align-items-center mb-2">
          <p class="lead mb-0 d-none" id="items-selected-info-message"></p>
          <div>
            <button class="btn btn-outline-secondary d-none mx-1" id="select-all-items-btn"></button>
            <button class="btn btn-outline-secondary d-none mx-1" id="clear-selection-btn">@L["ClearSelection"]</button>
            <button class="btn btn-danger mx-1" id="delete-selected-items">
              <i class="fa fa-trash"></i> @L["Delete"]
            </button>
          </div>
        </div>
        <hr class="my-1 mx-0" />
      </div>

      <table class="table table-striped dataTable" id="${entity_plural}Table">
        <thead>
          <tr>
            <th id="BulkDeleteCheckboxTheader">
              <input type="checkbox" id="select_all" class="form-check-input" />
            </th>
            <th>@L["Actions"].Value</th>
            ${vue_index_table_headers}
          </tr>
        </thead>
      </table>
    </div>
  </div>
</script>

<div id="${entity_plural_kebab}-page" v-cloak>
  <${entity_name_lower}-list-component></${entity_name_lower}-list-component>
</div>

