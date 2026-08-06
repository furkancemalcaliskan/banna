import { DOCUMENT, isPlatformBrowser } from '@angular/common';
import {
  ChangeDetectionStrategy,
  Component,
  inject,
  Injectable,
  PLATFORM_ID,
  signal,
} from '@angular/core';

type BannaAppearance = 'light' | 'dark';

@Injectable({ providedIn: 'root' })
export class ThemeAppearanceService {
  private readonly document = inject(DOCUMENT);
  private readonly platformId = inject(PLATFORM_ID);
  readonly appearance = signal<BannaAppearance>('dark');

  initialize(): void {
    if (!isPlatformBrowser(this.platformId)) return;

    const stored = this.readStoredAppearance();
    this.apply(stored === 'light' ? 'light' : 'dark', false);
  }

  toggle(): void {
    this.apply(this.appearance() === 'dark' ? 'light' : 'dark');
  }

  private apply(appearance: BannaAppearance, persist = true): void {
    this.appearance.set(appearance);
    if (!isPlatformBrowser(this.platformId)) return;

    const elements = [this.document.documentElement, this.document.body].filter(Boolean);
    for (const element of elements) {
      element.setAttribute('data-bs-theme', appearance);
      element.classList.remove('lpx-theme-light', 'lpx-theme-dark', 'lpx-theme-dim');
      element.classList.add(`lpx-theme-${appearance}`);
    }

    if (persist) this.storeAppearance(appearance);
  }

  private readStoredAppearance(): string | null {
    try {
      return window.localStorage.getItem('theme');
    } catch {
      return null;
    }
  }

  private storeAppearance(appearance: BannaAppearance): void {
    try {
      window.localStorage.setItem('theme', appearance);
    } catch {
      // Storage can be unavailable in privacy-restricted browser contexts.
    }
  }
}

@Component({
  selector: 'app-theme-changer',
  standalone: true,
  template: `
    <button
      type="button"
      class="btn btn-link nav-link banna-theme-changer"
      [attr.aria-label]="appearance.appearance() === 'dark' ? 'Use light theme' : 'Use dark theme'"
      [attr.title]="appearance.appearance() === 'dark' ? 'Use light theme' : 'Use dark theme'"
      (click)="appearance.toggle()"
    >
      <i
        class="fas"
        [class.fa-sun]="appearance.appearance() === 'dark'"
        [class.fa-moon]="appearance.appearance() === 'light'"
        aria-hidden="true"
      ></i>
    </button>
  `,
  styles: `
    :host { display: inline-flex; align-items: center; }
    .banna-theme-changer { min-width: 2.5rem; min-height: 2.5rem; }
  `,
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class ThemeChangerComponent {
  readonly appearance = inject(ThemeAppearanceService);
}
