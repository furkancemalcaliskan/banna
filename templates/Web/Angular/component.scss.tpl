.banna-datatable-footer {
  width: 100%;
  min-width: 720px;
  min-height: 50px;
  padding: 0.5rem 0.75rem;
  display: grid;
  grid-template-columns: minmax(15rem, 1fr) auto minmax(15rem, 1fr);
  align-items: center;
  gap: 1rem;
  color: var(--bs-secondary-color);
  font-size: 0.875rem;
}

.pager-size {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  white-space: nowrap;
}

.pager-size .form-select {
  width: auto;
  min-width: 5rem;
}

.pager-info {
  text-align: center;
  white-space: nowrap;
}

.pager-navigation {
  justify-self: end;
}

.pager-navigation .pagination {
  flex-wrap: nowrap;
  gap: 0.25rem;
}

.pager-navigation .page-link {
  border: 0;
  border-radius: 999px !important;
  box-shadow: 0 0.25rem 1rem rgba(var(--bs-body-color-rgb), 0.08);
  white-space: nowrap;
}

.pager-navigation .page-item.disabled .page-link {
  box-shadow: none;
}
