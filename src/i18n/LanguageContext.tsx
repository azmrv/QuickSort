import React, { createContext, useContext, useState, useCallback, useMemo } from 'react';
import { translations, type Locale } from './translations';

interface I18nContextValue {
    locale: Locale;
    setLocale: (l: Locale) => void;
    t: (key: string, params?: Record<string, string | number>) => string;
}

const LanguageContext = createContext<I18nContextValue | null>(null);

export const LanguageProvider: React.FC<{
    initialLocale: Locale;
    children: React.ReactNode;
}> = ({ initialLocale, children }) => {
    const [locale, setLocaleState] = useState<Locale>(initialLocale);

    const setLocale = useCallback((l: Locale) => {
        setLocaleState(l);
    }, []);

    const t = useCallback((key: string, params?: Record<string, string | number>): string => {
        const value = translations[locale]?.[key] || translations.en[key] || key;
        if (!params) return value;

        return Object.entries(params).reduce(
            (result, [paramKey, paramValue]) => result.replace(`{${paramKey}}`, String(paramValue)),
            value
        );
    }, [locale]);

    const value = useMemo(() => ({ locale, setLocale, t }), [locale, setLocale, t]);

    return (
        <LanguageContext.Provider value={value}>
            {children}
        </LanguageContext.Provider>
    );
};

export const useTranslation = (): I18nContextValue => {
    const context = useContext(LanguageContext);
    if (!context) {
        throw new Error('useTranslation must be used within a LanguageProvider');
    }
    return context;
};
