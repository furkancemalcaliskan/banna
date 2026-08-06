import { CommonModule } from '@angular/common';
import { Component, DestroyRef, OnInit, inject } from '@angular/core';
import { takeUntilDestroyed } from '@angular/core/rxjs-interop';
import {
  FormBuilder,
  FormGroup,
  FormsModule,
  ReactiveFormsModule,
  Validators,
} from '@angular/forms';

import {
  EnvironmentService,
  ListService,
  LocalizationPipe,
  LocalizationService,
  PagedResultDto,
  PermissionDirective,
} from '@abp/ng.core';
import {
  Confirmation,
  ConfirmationService,
  ThemeSharedModule,
  ToasterService,
} from '@abp/ng.theme.shared';

import { NgbDropdownModule } from '@ng-bootstrap/ng-bootstrap';

import { NgxDatatableModule, SelectionType } from '@swimlane/ngx-datatable';
import { Subject, debounceTime, distinctUntilChanged, finalize, switchMap } from 'rxjs';

import {
  __entity_name__ExcelDownloadDto,
  __entity_name__WithNavigationPropertiesDto,
  __entity_plural__Service,
  Get__entity_plural__Input,
} from '../proxy/__entity_plural_kebab__';

interface LookupDto<T> {
  id: T;
  displayName: string;
}

type __entity_name__Dto = __entity_name__WithNavigationPropertiesDto;

@Component({
  selector: 'app-__entity_name_kebab__',
  standalone: true,
  templateUrl: './__entity_name_kebab__.component.html',
  styleUrls: ['./__entity_name_kebab__.component.scss'],
  providers: [ListService],
  imports: [
    CommonModule,
    ThemeSharedModule,
    FormsModule,
    ReactiveFormsModule,
    NgxDatatableModule,
    NgbDropdownModule,
    LocalizationPipe,
    PermissionDirective,
  ],
})
export class __entity_name__Component implements OnInit {
  private readonly environment = inject(EnvironmentService);
  private readonly destroyRef = inject(DestroyRef);

  // List & filtering
  dto = { items: [], totalCount: 0 } as Required<PagedResultDto<__entity_name__Dto>>;
  filter = {} as Get__entity_plural__Input;

  // UI state
  form!: FormGroup;
  selectedObject = {} as __entity_name__Dto;
  isModalOpen = false;
  isDetailMode = false;
  isSaving = false;
  selectedRows: __entity_name__Dto[] = [];
  allItemsSelected = false;

  // UI helpers
  readonly pageSizes = [10, 25, 50, 100];
  readonly selectionType = SelectionType;
  advancedFilterOpen = false;

__nav_lookup_option_arrays__
__nav_lookup_search_subjects__

  constructor(
    public readonly list: ListService<Get__entity_plural__Input>,
    private readonly service: __entity_plural__Service,
    private readonly fb: FormBuilder,
    private readonly confirmation: ConfirmationService,
    private readonly localizationService: LocalizationService,
    private readonly toaster: ToasterService,
  ) {
    this.list.maxResultCount = 10;
  }

  ngOnInit(): void {
    const streamCreator = (query: Get__entity_plural__Input) =>
      this.service.getList({ ...query, ...this.filter });

    this.list.hookToQuery(streamCreator).subscribe(res => {
      this.dto = {
        items: res.items ?? [],
        totalCount: res.totalCount ?? 0,
      };
      this.clearSelection();
    });
__nav_lookup_search_bindings__
    this.loadAllLookups();
  }

  // ---------------------------
  // Filtering & paging
  // ---------------------------

  get totalPages(): number {
    return Math.max(1, Math.ceil(this.dto.totalCount / this.list.maxResultCount));
  }

  get currentPage(): number {
    return Math.min(this.list.page + 1, this.totalPages);
  }

  get pageStart(): number {
    return this.dto.totalCount === 0 ? 0 : this.list.page * this.list.maxResultCount + 1;
  }

  get pageEnd(): number {
    return Math.min((this.list.page + 1) * this.list.maxResultCount, this.dto.totalCount);
  }

  get visiblePages(): number[] {
    if (this.dto.totalCount === 0) return [];
    const visibleCount = Math.min(5, this.totalPages);
    const start = Math.max(1, Math.min(this.currentPage - 2, this.totalPages - visibleCount + 1));
    return Array.from({ length: visibleCount }, (_, index) => start + index);
  }

  changePageSize(): void {
    this.list.page = 0;
    this.list.get();
  }

  goToPage(page: number): void {
    const target = Math.min(Math.max(1, page), this.totalPages);
    if (target === this.currentPage) return;
    this.list.page = target - 1;
    this.list.get();
  }

  toggleFilter(): void {
    this.advancedFilterOpen = !this.advancedFilterOpen;
  }

  applyFilter(): void {
    this.refreshListFirstPage();
  }

  clearFilter(): void {
    this.filter = {} as Get__entity_plural__Input;
    this.refreshListFirstPage();
  }

  private refreshListFirstPage(): void {
    this.clearSelection();
    this.list.page = 0;
    this.list.get();
  }

  // ---------------------------
  // Modal operations
  // ---------------------------

  create(): void {
    this.openModal({ dto: {} as __entity_name__Dto, detail: false });
  }

  edit(id: string): void {
    this.openById(id, false);
  }

  detail(id: string): void {
    this.openById(id, true);
  }

  private openById(id: string, detail: boolean): void {
    this.service.getWithNavigationProperties(id).subscribe(dto => {
      this.openModal({ dto, detail });
    });
  }

  private openModal(args: { dto: __entity_name__Dto; detail: boolean }): void {
    this.selectedObject = args.dto;
    this.isDetailMode = args.detail;

    this.buildForm();
    this.isModalOpen = true;
  }

  // ---------------------------
  // Form
  // ---------------------------

  buildForm(): void {
    const c = this.selectedObject?.__entity_name_lower__;

    this.form = this.fb.group({
__form_group_entries__
    });

    this.setFormMode(this.isDetailMode);
  }

  private setFormMode(isDetail: boolean): void {
    if (!this.form) return;
    if (isDetail) this.form.disable();
    else this.form.enable();
  }

  save(): void {
    if (!this.form || this.form.invalid || this.isDetailMode || this.isSaving) return;

    const raw = this.form.getRawValue();
    const id = this.selectedObject.__entity_name_lower__?.id;
    const payload = {
      ...raw,
__payload_mappings__
      ...(id
        ? { concurrencyStamp: this.selectedObject.__entity_name_lower__?.concurrencyStamp }
        : {}),
    };

    const request$ = id ? this.service.update(id, payload) : this.service.create(payload);

    this.isSaving = true;
    request$.pipe(finalize(() => (this.isSaving = false))).subscribe(() => {
      this.closeModal();
      this.toaster.success('::SuccessfullySaved');
      this.list.get();
    });
  }

  closeModal(): void {
    this.isModalOpen = false;
    this.form?.reset();
    this.selectedObject = {} as __entity_name__Dto;
    this.isDetailMode = false;
  }

  delete(id: string): void {
    this.confirmation.warn('::AreYouSureToDelete', 'AbpAccount::AreYouSure').subscribe(status => {
      if (status !== Confirmation.Status.confirm) return;
      this.service.delete(id).subscribe(() => {
        this.toaster.success('::SuccessfullyDeleted');
        this.list.get();
      });
    });
  }

  onSelect(event: { selected?: __entity_name__Dto[] }): void {
    this.selectedRows = [...(event.selected ?? [])];
    this.allItemsSelected = false;
  }

  selectAllItems(): void {
    if (!this.selectedRows.length) return;
    this.allItemsSelected = true;
  }

  clearSelection(): void {
    this.selectedRows = [];
    this.allItemsSelected = false;
  }

  deleteSelected(): void {
    if (!this.selectedRows.length) return;

    const count = this.allItemsSelected ? this.dto.totalCount : this.selectedRows.length;
    const message = this.allItemsSelected
      ? '::DeleteAllRecords'
      : this.localizationService.instant('::DeleteSelectedRecords', count.toString());

    this.confirmation.warn(message, 'AbpAccount::AreYouSure').subscribe(status => {
      if (status !== Confirmation.Status.confirm) return;

      const request$ = this.allItemsSelected
        ? this.service.deleteAll(this.filter)
        : this.service.deleteByIds(
            this.selectedRows
              .map(row => row.__entity_name_lower__?.id)
              .filter((id): id is string => !!id),
          );

      request$.subscribe(() => {
        this.toaster.success('::SuccessfullyDeleted');
        this.clearSelection();
        this.refreshListFirstPage();
      });
    });
  }

  private loadAllLookups(): void {
__nav_lookup_load_calls__
  }

__nav_lookup_helpers__

  // ---------------------------
  // Excel export
  // ---------------------------

  exportToExcel(): void {
    this.service.getDownloadToken().subscribe(res => {
      if (!res.token) return;
      const apiBase = this.normalizeBaseUrl(this.environment.getApiUrl(undefined));
      const input = this.getFilterForExcel();
      const queryString = this.buildExcelQuery(res.token, input);

      const url = `${apiBase}/api/app/__entity_plural_kebab__/as-excel-file${queryString}`;
      this.openInNewTab(url);
    });
  }

  private openInNewTab(url: string): void {
    const a = document.createElement('a');
    a.href = url;
    a.target = '_blank';
    a.rel = 'noopener';
    document.body.appendChild(a);
    a.click();
    a.remove();
  }

  private getFilterForExcel(): __entity_name__ExcelDownloadDto {
    const f = this.filter;
    return {
__excel_filter_body__
    };
  }

  private buildExcelQuery(
    downloadToken: string,
    input: __entity_name__ExcelDownloadDto,
  ): string {
    const qs = new URLSearchParams();
    qs.set('DownloadToken', downloadToken);

__excel_query_appends__
    const query = qs.toString();
    return query ? `?${query}` : '';
  }

  private appendIfHasValue(qs: URLSearchParams, name: string, value: unknown): void {
    if (value === undefined || value === null) return;
    if (typeof value === 'string' && value.trim() === '') return;
    qs.append(name, String(value));
  }

  // ---------------------------
  // Date helpers
  // ---------------------------

  private toDatetimeLocal(value: unknown): string | null {
    const d = this.toDateSafe(value);
    if (!d) return null;

    const pad = (n: number) => String(n).padStart(2, '0');
    return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}T${pad(
      d.getHours(),
    )}:${pad(d.getMinutes())}`;
  }

  private toIsoStringSafe(value: unknown): string | null {
    const d = this.toDateSafe(value);
    return d ? d.toISOString() : null;
  }

  private toDateSafe(value: unknown): Date | null {
    if (!value) return null;
    const d = value instanceof Date ? value : new Date(String(value));
    return isNaN(d.getTime()) ? null : d;
  }

  private normalizeBaseUrl(url: string): string {
    if (!url) return '';
    return url.endsWith('/') ? url.slice(0, -1) : url;
  }
}
