import { APP_INITIALIZER, Provider, inject, provideAppInitializer } from '@angular/core';
import { NavItemsService } from '@abp/ng.theme.shared';
import { ThemeAppearanceService, ThemeChangerComponent } from './theme-changer.component';

const registerThemeChanger = (
  appearance: ThemeAppearanceService,
  navItems: NavItemsService,
): void => {
  appearance.initialize();
  if (!navItems.items.some(item => item.id === 'Banna.ThemeChanger')) {
    navItems.addItems([
      {
        id: 'Banna.ThemeChanger',
        order: 90,
        component: ThemeChangerComponent,
      },
    ]);
  }
};

export const provideThemeChanger = () =>
  provideAppInitializer(() =>
    registerThemeChanger(inject(ThemeAppearanceService), inject(NavItemsService)),
  );

// Kept for NgModule-based ABP Angular applications.
export const THEME_CHANGER_PROVIDER: Provider = {
  provide: APP_INITIALIZER,
  multi: true,
  deps: [ThemeAppearanceService, NavItemsService],
  useFactory: (appearance: ThemeAppearanceService, navItems: NavItemsService) => () =>
    registerThemeChanger(appearance, navItems),
};
