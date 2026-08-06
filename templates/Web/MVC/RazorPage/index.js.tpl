$(function () {
  var l = abp.localization.getResource("${domain_short}");

  if ($.fn.select2) {
    $('.select2').select2({ width: '100%' });
  }

  ${index_js_select2_nav_filters}

  var ${entity_name_lower}Service = window.${namespace_lower}.${entity_plural_lower};

  var createModal = new abp.ModalManager({
    viewUrl: abp.appPath + "${entity_plural}/CreateModal",
    scriptUrl: abp.appPath + "Pages/${entity_plural}/createModal.js",
    modalClass: "${entity_name_lower}Create"
  });

  var editModal = new abp.ModalManager({
    viewUrl: abp.appPath + "${entity_plural}/EditModal",
    scriptUrl: abp.appPath + "Pages/${entity_plural}/editModal.js",
    modalClass: "${entity_name_lower}Edit"
  });

  var detailModal = new abp.ModalManager({
    viewUrl: abp.appPath + "${entity_plural}/DetailModal",
    scriptUrl: abp.appPath + "Pages/${entity_plural}/detailModal.js",
    modalClass: "${entity_name_lower}Detail"
  });

  var getFilter = function () {
    return {
      filterText: $("#FilterText").val(),
      ${index_js_string_filters}
      ${index_js_nav_filters}
    };
  };

  var dataTableColumns = [
    {
      rowAction: {
        items: [
          {
            text: l("Detail"),
            visible: abp.auth.isGranted('${domain_short}.${entity_plural}.Detail'),
            action: function (data) {
              detailModal.open({ id: data.record.${entity_name_lower}.id });
            }
          },
          {
            text: l("Edit"),
            visible: abp.auth.isGranted('${domain_short}.${entity_plural}.Edit'),
            action: function (data) {
              editModal.open({ id: data.record.${entity_name_lower}.id });
            }
          },
          {
            text: l("Delete"),
            visible: abp.auth.isGranted('${domain_short}.${entity_plural}.Delete'),
            confirmMessage: function () { return l("DeleteConfirmationMessage"); },
            action: function (data) {
              ${entity_name_lower}Service.delete(data.record.${entity_name_lower}.id).then(function () {
                abp.notify.success(l("SuccessfullyDeleted"));
                dataTable.ajax.reloadEx();
              });
            }
          }
        ]
      },
      width: "1rem"
    },
    ${web_datatable_columns}
  ];

  if (abp.auth.isGranted('${domain_short}.${entity_plural}.Delete')) {
    dataTableColumns.unshift({
      targets: 0,
      data: null,
      orderable: false,
      className: 'select-checkbox',
      width: "0.5rem",
      render: function (data) {
        return '<input type="checkbox" class="form-check-input select-row-checkbox" data-id="' + data.${entity_name_lower}.id + '"/>';
      }
    });
  } else {
    $("#BulkDeleteCheckboxTheader").remove();
  }

  var dataTable = $("#${entity_plural}Table").DataTable(abp.libs.datatables.normalizeConfiguration({
    processing: true,
    serverSide: true,
    paging: true,
    searching: false,
    scrollX: true,
    autoWidth: true,
    scrollCollapse: true,
    order: [[2, "asc"]],
    ajax: abp.libs.datatables.createAjax(${entity_name_lower}Service.getList, getFilter),
    columnDefs: dataTableColumns
  }));

  dataTable.on("xhr", function () {
    selectOrUnselectAllCheckboxes(false);
    showOrHideContextMenu();
    $("#select_all").prop("indeterminate", false).prop("checked", false);
  });

  function selectOrUnselectAllCheckboxes(selectAll) {
    $(".select-row-checkbox").each(function () { $(this).prop("checked", selectAll); });
  }

  $("#select_all").click(function () {
    if ($(this).is(":checked")) selectOrUnselectAllCheckboxes(true);
    else selectOrUnselectAllCheckboxes(false);
    showOrHideContextMenu();
  });

  dataTable.on("change", "input[type='checkbox'].select-row-checkbox", function () {
    var unSelected = $("input[type='checkbox'].select-row-checkbox:not(:checked)");
    if (unSelected.length >= 1) {
      var dataRecordTotal = dataTable.context[0].json.data.length;
      if (unSelected.length === dataRecordTotal) {
        $("#select_all").prop("indeterminate", false).prop("checked", false);
      } else {
        $("#select_all").prop("indeterminate", true);
      }
    } else {
      $("#select_all").prop("indeterminate", false).prop("checked", true);
    }
    showOrHideContextMenu();
  });

  var showOrHideContextMenu = function () {
    var selected = $("input[type='checkbox'].select-row-checkbox:is(:checked)");
    var selectedCount = selected.length;
    var dataRecordTotal = dataTable.context[0].json.data.length;
    var recordsTotal = dataTable.context[0].json.recordsTotal;

    if (selectedCount >= 1) {
      $("#bulk-delete-context-menu").removeClass("d-none");
      $("#items-selected-info-message")
        .html(selectedCount === 1 ? l("OneItemOnThisPageIsSelected") : l("NumberOfItemsOnThisPageAreSelected", selectedCount))
        .removeClass("d-none");

      if (selectedCount === dataRecordTotal && recordsTotal > dataRecordTotal) {
        $("#select-all-items-btn").html(l("SelectAllItems", recordsTotal)).removeClass("d-none");
        $("#select-all-items-btn").off("click").click(function () {
          $(this).data("selected", true).addClass("d-none");
          $("#items-selected-info-message").html(l("AllItemsAreSelected", recordsTotal));
          $("#clear-selection-btn").removeClass("d-none");
        });

        $("#clear-selection-btn").off("click").click(function () {
          $("#select-all-items-btn").data("selected", false);
          $("#select_all").prop("checked", false);
          selectOrUnselectAllCheckboxes(false);
          showOrHideContextMenu();
        });
      } else {
        $("#select-all-items-btn").addClass("d-none").data("selected", false);
        $("#clear-selection-btn").addClass("d-none");
      }

      $("#delete-selected-items").off("click").click(function () {
        if ($("#select-all-items-btn").data("selected") === true) {
          abp.message.confirm(l("DeleteAllRecords"), function (confirmed) {
            if (!confirmed) return;
            ${entity_name_lower}Service.deleteAll(getFilter()).then(function () {
              dataTable.ajax.reloadEx();
              selectOrUnselectAllCheckboxes(false);
              showOrHideContextMenu();
            });
          });
        } else {
          var ids = [];
          $("input[type='checkbox'].select-row-checkbox:is(:checked)").each(function () { ids.push($(this).data("id")); });
          abp.message.confirm(l("DeleteSelectedRecords", ids.length), function (confirmed) {
            if (!confirmed) return;
            ${entity_name_lower}Service.deleteByIds(ids).then(function () {
              dataTable.ajax.reloadEx();
              selectOrUnselectAllCheckboxes(false);
              showOrHideContextMenu();
            });
          });
        }
      });
    } else {
      $("#bulk-delete-context-menu, #select-all-items-btn, #items-selected-info-message, #clear-selection-btn").addClass("d-none");
    }
  };

  createModal.onResult(function () {
    dataTable.ajax.reloadEx();
    selectOrUnselectAllCheckboxes(false);
    showOrHideContextMenu();
  });

  editModal.onResult(function () {
    dataTable.ajax.reloadEx();
    selectOrUnselectAllCheckboxes(false);
    showOrHideContextMenu();
  });

  detailModal.onResult(function () {
    dataTable.ajax.reloadEx();
    selectOrUnselectAllCheckboxes(false);
    showOrHideContextMenu();
  });

  $("#New${entity_name}Button").click(function (e) {
    e.preventDefault();
    createModal.open();
  });

  $("#SearchForm").submit(function (e) {
    e.preventDefault();
    dataTable.ajax.reloadEx();
    selectOrUnselectAllCheckboxes(false);
    showOrHideContextMenu();
  });

  $("#ExportToExcelButton").click(function (e) {
    e.preventDefault();
    ${entity_name_lower}Service.getDownloadToken().then(function (result) {
      var input = getFilter();
      var url = abp.appPath + 'api/app/${entity_plural_kebab}/as-excel-file' +
        abp.utils.buildQueryString([
          { name: 'downloadToken', value: result.token },
          { name: 'filterText', value: input.filterText },
          ${index_js_string_query_pairs}
          ${index_js_nav_query_pairs}
        ]);
      var w = window.open(url, '_blank'); if (w) w.focus();
    });
  });

  $('#AdvancedFilterSectionToggler').on('click', function () {
    $('#AdvancedFilterSection').toggle();
    var iconCss = $("#AdvancedFilterSection").is(":visible") ? "fa ms-1 fa-angle-up" : "fa ms-1 fa-angle-down";
    $(this).find("i").attr("class", iconCss);
  });

  $('#AdvancedFilterSection').on('keypress', function (e) {
    if (e.which === 13) {
      dataTable.ajax.reloadEx();
      selectOrUnselectAllCheckboxes(false);
      showOrHideContextMenu();
    }
  });

  $('#AdvancedFilterSection select').change(function () {
    dataTable.ajax.reloadEx();
    selectOrUnselectAllCheckboxes(false);
    showOrHideContextMenu();
  });

  $("#ClearFiltersButton").click(function () {
    $("#FilterText").val("");
    ${index_js_string_clear_filters}
    ${index_js_nav_clear_filters}
    dataTable.ajax.reloadEx();
  });
});
