const { createApp, ref, watch, onMounted } = Vue;
const ${entity_name_lower}Service = window.${namespace_lower}.${entity_plural_lower};
const l = abp.localization.getResource("${domain_short}");

const ${entity_name}ListComponent = {
  template: "#${entity_name_lower}-list-template",
  setup() {
    const dataTable = ref(null);
    const createModal = ref(new abp.ModalManager({
      viewUrl: abp.appPath + "${entity_plural}/CreateModal",
      scriptUrl: abp.appPath + "Pages/${entity_plural}/createModal.js",
      modalClass: "${entity_name_lower}Create"
    }));
    const editModal = ref(new abp.ModalManager({
      viewUrl: abp.appPath + "${entity_plural}/EditModal",
      scriptUrl: abp.appPath + "Pages/${entity_plural}/editModal.js",
      modalClass: "${entity_name_lower}Edit"
    }));
    const detailModal = ref(new abp.ModalManager({
      viewUrl: abp.appPath + "${entity_plural}/DetailModal",
      scriptUrl: abp.appPath + "Pages/${entity_plural}/detailModal.js",
      modalClass: "${entity_name_lower}Detail"
    }));

    const filter = ref({
      filterText: "",
      ${vue_index_filter_fields}
    });

    ${vue_index_support_refs}
    ${vue_index_lookup_functions}

    const showAdvancedFilter = ref(false);
    function toggleAdvancedFilter(){ showAdvancedFilter.value = !showAdvancedFilter.value; }

    function getFilter(){ return filter.value; }
    let filterReloadTimer = null;
    function reloadTable() {
      if (filterReloadTimer) clearTimeout(filterReloadTimer);
      filterReloadTimer = null;
      dataTable.value?.ajax.reloadEx();
    }
    function scheduleFilterReload() {
      if (filterReloadTimer) clearTimeout(filterReloadTimer);
      filterReloadTimer = setTimeout(reloadTable, 300);
    }

    onMounted(async () => {
      initToolbar();
      initTable();
      ${vue_index_onmounted_fetches}
    });

    function initToolbar() {
      const exportBtn = document.getElementById("ExportToExcelButton");
      if (exportBtn) exportBtn.addEventListener("click", onExportExcel);
      const newBtn = document.getElementById("New${entity_name}Button");
      if (newBtn) newBtn.addEventListener("click", e => { e.preventDefault(); createModal.value.open(); });
    }

    function initTable() {
      const columns = [
        {
          rowAction: {
            items: [
              { text: l("Detail"),  visible: abp.auth.isGranted('${domain_short}.${entity_plural}.Detail'), action: d => detailModal.value.open({ id: d.record.${entity_name_lower}.id }) },
              { text: l("Edit"),    visible: abp.auth.isGranted('${domain_short}.${entity_plural}.Edit'),   action: d => editModal.value.open({ id: d.record.${entity_name_lower}.id }) },
              { text: l("Delete"),  visible: abp.auth.isGranted('${domain_short}.${entity_plural}.Delete'),
                confirmMessage: () => l("DeleteConfirmationMessage"),
                action: d => { ${entity_name_lower}Service.delete(d.record.${entity_name_lower}.id).then(() => { abp.notify.success(l("SuccessfullyDeleted")); dataTable.value.ajax.reloadEx(); }); } }
            ]
          },
          width: "1rem"
        },
        ${web_datatable_columns}
      ];

      if (abp.auth.isGranted('${domain_short}.${entity_plural}.Delete')) {
        columns.unshift({
          targets: 0, data: null, orderable: false, className: 'select-checkbox', width: "0.5rem",
          render: data => '<input type="checkbox" class="form-check-input select-row-checkbox" data-id="' + data.${entity_name_lower}.id + '"/>'
        });
      } else {
        const th = document.getElementById("BulkDeleteCheckboxTheader");
        if (th && th.parentElement) th.parentElement.removeChild(th);
      }

      const table = document.getElementById("${entity_plural}Table");
      dataTable.value = new DataTable(table, abp.libs.datatables.normalizeConfiguration({
        processing: true, serverSide: true, paging: true, searching: false,
        scrollX: true, autoWidth: true, scrollCollapse: true,
        order: [[abp.auth.isGranted('${domain_short}.${entity_plural}.Delete') ? 2 : 1, "asc"]],
        ajax: abp.libs.datatables.createAjax(${entity_name_lower}Service.getList, getFilter),
        columnDefs: columns
      }));

      dataTable.value.on("xhr", function() {
        selectOrUnselectAllCheckboxes(false);
        showOrHideContextMenu();
        const selectAll = document.getElementById("select_all");
        if (selectAll) {
          selectAll.indeterminate = false;
          selectAll.checked = false;
        }
      });

      function selectOrUnselectAllCheckboxes(selectAll) {
        document
          .querySelectorAll("input.form-check-input.select-row-checkbox")
          .forEach(cb => { cb.checked = !!selectAll; });
      }

      const selectAllEl = document.getElementById("select_all");
      if (selectAllEl) {
        selectAllEl.addEventListener("click", function() {
          if (this.checked) selectOrUnselectAllCheckboxes(true);
          else selectOrUnselectAllCheckboxes(false);
          showOrHideContextMenu();
        });
      }

      dataTable.value.on("change", "input[type='checkbox'].select-row-checkbox", function() {
        const unSelected = table.querySelectorAll("input[type='checkbox'].select-row-checkbox:not(:checked)");
        const selectAll = document.getElementById("select_all");
        if (selectAll) {
          if (unSelected.length >= 1) {
            const dataRecordTotal = dataTable.value.ajax.json()?.data?.length ?? 0;
            if (unSelected.length === dataRecordTotal) {
              selectAll.indeterminate = false;
              selectAll.checked = false;
            } else {
              selectAll.indeterminate = true;
            }
          } else {
            selectAll.indeterminate = false;
            selectAll.checked = true;
          }
        }
        showOrHideContextMenu();
      });

      function showOrHideContextMenu() {
        const selected = table.querySelectorAll("input[type='checkbox'].select-row-checkbox:checked");
        const selectedCount = selected.length;

        const ctx = document.getElementById("bulk-delete-context-menu");
        const info = document.getElementById("items-selected-info-message");
        const selectAllBtn = document.getElementById("select-all-items-btn");
        const clearBtn = document.getElementById("clear-selection-btn");
        const deleteBtn = document.getElementById("delete-selected-items");

        if (!ctx || !info || !selectAllBtn || !clearBtn || !deleteBtn) return;

        const response = dataTable.value.ajax.json() ?? {};
        const dataRecordTotal = response.data?.length ?? 0;
        const recordsTotal = response.recordsTotal ?? dataRecordTotal;

        if (selectedCount >= 1) {
          ctx.classList.remove("d-none");
          info.textContent = (selectedCount === 1)
            ? l("OneItemOnThisPageIsSelected")
            : l("NumberOfItemsOnThisPageAreSelected", selectedCount);
          info.classList.remove("d-none");

          if (selectedCount === dataRecordTotal && recordsTotal > dataRecordTotal) {
            selectAllBtn.textContent = l("SelectAllItems", recordsTotal);
            selectAllBtn.classList.remove("d-none");
            selectAllBtn.onclick = function() {
              this.dataset.selected = "true";
              this.classList.add("d-none");
              info.textContent = l("AllItemsAreSelected", recordsTotal);
              clearBtn.classList.remove("d-none");
            };

            clearBtn.onclick = function() {
              selectAllBtn.dataset.selected = "false";
              const selAll = document.getElementById("select_all");
              if (selAll) selAll.checked = false;
              selectOrUnselectAllCheckboxes(false);
              showOrHideContextMenu();
            };
          } else {
            selectAllBtn.classList.add("d-none");
            selectAllBtn.dataset.selected = "false";
            clearBtn.classList.add("d-none");
            clearBtn.onclick = null;
          }

          deleteBtn.onclick = function() {
            const selectAllChosen = (selectAllBtn.dataset.selected === "true");
            if (selectAllChosen) {
              abp.message.confirm(l("DeleteAllRecords"), function(confirmed) {
                if (!confirmed) return;
                ${entity_name_lower}Service.deleteAll(getFilter()).then(function() {
                  dataTable.value.ajax.reloadEx();
                  selectOrUnselectAllCheckboxes(false);
                  showOrHideContextMenu();
                });
              });
            } else {
              const ids = Array.from(
                table.querySelectorAll("input[type='checkbox'].select-row-checkbox:checked")
              ).map(cb => cb.getAttribute("data-id"));
              abp.message.confirm(l("DeleteSelectedRecords", ids.length), function(confirmed) {
                if (!confirmed) return;
                ${entity_name_lower}Service.deleteByIds(ids).then(function() {
                  dataTable.value.ajax.reloadEx();
                  selectOrUnselectAllCheckboxes(false);
                  showOrHideContextMenu();
                });
              });
            }
          };
        } else {
          ["bulk-delete-context-menu", "select-all-items-btn", "items-selected-info-message", "clear-selection-btn"]
            .forEach(id => {
              const el = document.getElementById(id);
              if (el) el.classList.add("d-none");
            });
        }
      }
    }

    watch(() => filter.value.filterText, v => { if (!v) scheduleFilterReload(); });

    ${vue_index_watch_filters}

    createModal.value.onResult(() => dataTable.value.ajax.reloadEx());
    editModal.value.onResult(() => dataTable.value.ajax.reloadEx());
    detailModal.value.onResult(() => dataTable.value.ajax.reloadEx());

    function onExportExcel(e) {
      e.preventDefault();
      ${entity_name_lower}Service.getDownloadToken().then(result => {
        const url = abp.appPath + 'api/app/${entity_plural_kebab}/as-excel-file' +
          abp.utils.buildQueryString([
            { name: 'downloadToken', value: result.token },
            { name: 'filterText', value: filter.value.filterText },
            ${index_js_string_query_pairs}
            ${index_js_nav_query_pairs}
          ]);
        const w = window.open(url, '_blank'); w?.focus();
      });
    }

    function onGetFilteredData(){ reloadTable(); }

    function clearFilters(){
      filter.value = { ${vue_index_clear_filter_defaults} };
      scheduleFilterReload();
    }

    return { filter, showAdvancedFilter, toggleAdvancedFilter, onGetFilteredData, clearFilters${vue_index_return_refs} };
  }
};

const ${entity_plural_lower}App = createApp({ components: { "${entity_name_lower}-list-component": ${entity_name}ListComponent }});
if (window["vue-select"]) {
  ${entity_plural_lower}App.component("v-select", window["vue-select"]);
}
${entity_plural_lower}App.mount("#${entity_plural_kebab}-page");
