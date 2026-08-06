import { ComponentFixture, TestBed } from '@angular/core/testing';

import { __entity_name__Component } from './__entity_name_kebab__.component';

describe('__entity_name__Component', () => {
  let component: __entity_name__Component;
  let fixture: ComponentFixture<__entity_name__Component>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [__entity_name__Component],
    }).compileComponents();

    fixture = TestBed.createComponent(__entity_name__Component);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
