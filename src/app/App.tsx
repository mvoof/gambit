import { useTranslation } from 'react-i18next';

export const App = () => {
  const { t } = useTranslation();

  return <h1>{t('app.title')}</h1>;
};
