import React from 'react';

type LocalizationValue = {
  t: (key: string, ...args: any[]) => string;
  locale: string;
};

export const LocalizationContext = React.createContext<LocalizationValue | null>(null);
