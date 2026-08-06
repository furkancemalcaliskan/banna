<div class="row align-items-center mb-3">
  <div class="col-md-6 d-flex align-items-center">
    <h4 class="mb-0">{{ '::Menu:__entity_plural__' | abpLocalization }}</h4>
  </div>

  <div class="col-md-6 text-end">
    <button class="btn btn-sm btn-success me-2" type="button" (click)="exportToExcel()">
      <i class="fa fa-file-excel me-1"></i>
      <span>{{ '::ExportToExcel' | abpLocalization }}</span>
    </button>

    <button
      *abpPermission="'__domain_short__.__entity_plural__.Create'"
      id="create"
      class="btn btn-sm btn-primary"
      type="button"
      (click)="create()"
    >
      <i class="fa fa-plus me-1"></i>
      <span>{{ '::New__entity_name__' | abpLocalization }}</span>
    </button>
  </div>
</div>

<div class="card">
  <div class="card-body">
    <div class="form-group mb-3">
      <div class="input-group">
        <input
          class="form-control"
          [placeholder]="'::Search' | abpLocalization"
          (keyup.enter)="applyFilter()"
          [(ngModel)]="filter.filterText"
        />

        <button class="btn btn-outline-primary" type="button" (click)="applyFilter()">
          <i class="fa fa-search"></i>
        </button>
        <button class="btn btn-outline-success" type="button" (click)="toggleFilter()">
          <i class="fa fa-filter"></i>
        </button>
        <button class="btn btn-outline-danger" type="button" (click)="clearFilter()">
          <i class="fa fa-times"></i>
        </button>
      </div>
    </div>

    <div class="row g-2 mb-3" *ngIf="advancedFilterOpen">
__advanced_filter_inputs__
    </div>

    <ng-container *abpPermission="'__domain_short__.__entity_plural__.Delete'">
      <div
        class="alert alert-info d-flex flex-wrap align-items-center gap-2"
        *ngIf="selectedRows.length"
      >
        <span>
          {{
            allItemsSelected
              ? ('::AllItemsAreSelected' | abpLocalization: [dto.totalCount.toString()])
              : selectedRows.length === 1
                ? ('::OneItemOnThisPageIsSelected' | abpLocalization)
                : ('::NumberOfItemsOnThisPageAreSelected'
                  | abpLocalization: [selectedRows.length.toString()])
          }}
        </span>

        <button
          *ngIf="!allItemsSelected && selectedRows.length === dto.items.length && dto.totalCount > dto.items.length"
          class="btn btn-link p-0"
          type="button"
          (click)="selectAllItems()"
        >
          {{ '::SelectAllItems' | abpLocalization: [dto.totalCount.toString()] }}
        </button>

        <button class="btn btn-link p-0" type="button" (click)="clearSelection()">
          {{ '::ClearSelection' | abpLocalization }}
        </button>

        <button class="btn btn-sm btn-danger ms-auto" type="button" (click)="deleteSelected()">
          <i class="fa fa-trash me-1"></i>
          {{ '::Delete' | abpLocalization }}
        </button>
      </div>
    </ng-container>

    <ngx-datatable
      [rows]="dto.items"
      [count]="dto.totalCount"
      [list]="list"
      [externalPaging]="true"
      [externalSorting]="true"
      [footerHeight]="50"
      [limit]="list.maxResultCount"
      [offset]="list.page"
      [selected]="selectedRows"
      [selectionType]="selectionType.checkbox"
      (select)="onSelect($event)"
      default
    >
      <ngx-datatable-column
        *abpPermission="'__domain_short__.__entity_plural__.Delete'"
        [width]="42"
        [maxWidth]="42"
        [sortable]="false"
        [canAutoResize]="false"
        [draggable]="false"
        [resizeable]="false"
        [headerCheckboxable]="true"
        [checkboxable]="true"
      ></ngx-datatable-column>

      <ngx-datatable-column
        [name]="'::Actions' | abpLocalization"
        [maxWidth]="180"
        [sortable]="false"
      >
        <ng-template let-row="row" ngx-datatable-cell-template>
          <div ngbDropdown container="body" class="d-inline-block">
            <button class="btn btn-primary btn-sm dropdown-toggle" ngbDropdownToggle>
              <i class="fa fa-cog me-1"></i>{{ '::Actions' | abpLocalization }}
            </button>

            <div ngbDropdownMenu>
              <button
                *abpPermission="'__domain_short__.__entity_plural__.Detail'"
                ngbDropdownItem
                (click)="detail(row.__entity_name_lower__.id)"
              >
                {{ '::Detail' | abpLocalization }}
              </button>

              <button
                *abpPermission="'__domain_short__.__entity_plural__.Edit'"
                ngbDropdownItem
                (click)="edit(row.__entity_name_lower__.id)"
              >
                {{ '::Edit' | abpLocalization }}
              </button>

              <button
                *abpPermission="'__domain_short__.__entity_plural__.Delete'"
                ngbDropdownItem
                (click)="delete(row.__entity_name_lower__.id)"
              >
                {{ '::Delete' | abpLocalization }}
              </button>
            </div>
          </div>
        </ng-template>
      </ngx-datatable-column>

__datatable_columns__

      <ngx-datatable-footer>
        <ng-template ngx-datatable-footer-template>
          <div class="banna-datatable-footer">
            <div class="pager-size">
              <span>{{ '::PagerShow' | abpLocalization }}</span>
              <select
                class="form-select form-select-sm"
                [attr.aria-label]="'::PagerPageSize' | abpLocalization"
                [(ngModel)]="list.maxResultCount"
                (ngModelChange)="changePageSize()"
              >
                <option *ngFor="let size of pageSizes" [ngValue]="size">{{ size }}</option>
              </select>
              <span>{{ '::PagerEntries' | abpLocalization }}</span>
            </div>

            <div class="pager-info" aria-live="polite">
              {{
                '::PagerInfo'
                  | abpLocalization: [pageStart.toString(), pageEnd.toString(), dto.totalCount.toString()]
              }}
            </div>

            <nav class="pager-navigation" [attr.aria-label]="'::Pagination' | abpLocalization">
              <ul class="pagination pagination-sm mb-0">
                <li class="page-item" [class.disabled]="currentPage === 1">
                  <button class="page-link" type="button" [disabled]="currentPage === 1" (click)="goToPage(1)">
                    {{ '::PagerFirst' | abpLocalization }}
                  </button>
                </li>
                <li class="page-item" [class.disabled]="currentPage === 1">
                  <button class="page-link" type="button" [disabled]="currentPage === 1" (click)="goToPage(currentPage - 1)">
                    {{ '::PagerPrevious' | abpLocalization }}
                  </button>
                </li>
                <li *ngFor="let page of visiblePages" class="page-item" [class.active]="page === currentPage">
                  <button
                    class="page-link"
                    type="button"
                    [attr.aria-current]="page === currentPage ? 'page' : null"
                    (click)="goToPage(page)"
                  >
                    {{ page }}
                  </button>
                </li>
                <li class="page-item" [class.disabled]="currentPage === totalPages">
                  <button class="page-link" type="button" [disabled]="currentPage === totalPages" (click)="goToPage(currentPage + 1)">
                    {{ '::PagerNext' | abpLocalization }}
                  </button>
                </li>
                <li class="page-item" [class.disabled]="currentPage === totalPages">
                  <button class="page-link" type="button" [disabled]="currentPage === totalPages" (click)="goToPage(totalPages)">
                    {{ '::PagerLast' | abpLocalization }}
                  </button>
                </li>
              </ul>
            </nav>
          </div>
        </ng-template>
      </ngx-datatable-footer>

    </ngx-datatable>
  </div>
</div>

<abp-modal [(visible)]="isModalOpen" [options]="{ size: 'lg', centered: true }">
  <ng-template #abpHeader>
    <h3>
      {{
        (selectedObject?.__entity_name_lower__?.id ? (isDetailMode ? '::Detail' : '::Edit') : '::New__entity_name__')
          | abpLocalization
      }}
    </h3>
  </ng-template>

  <ng-template #abpBody>
    <form [formGroup]="form" (ngSubmit)="save()">
__modal_form_fields__
    </form>
  </ng-template>

  <ng-template #abpFooter>
    <button type="button" class="btn btn-secondary" (click)="closeModal()">
      {{ '::Close' | abpLocalization }}
    </button>

    <button
      type="button"
      class="btn btn-primary"
      (click)="save()"
      [disabled]="form.invalid || isSaving"
      *ngIf="!isDetailMode"
    >
      <i class="fa fa-check mr-1"></i>
      {{ '::Save' | abpLocalization }}
    </button>
  </ng-template>
</abp-modal>
