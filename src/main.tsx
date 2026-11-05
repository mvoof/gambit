import '@fontsource-variable/jetbrains-mono/index.css';
import '@fontsource-variable/inter/index.css';
import '@fontsource-variable/source-serif-4/index.css';
import React from 'react';
import ReactDOM from 'react-dom/client';

import { App } from '@app/App';

import './i18n';
import './styles/index.scss';

const rootElement = document.getElementById('root');

if (!rootElement) {
  throw new Error('Root element #root is missing from index.html');
}

ReactDOM.createRoot(rootElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>
);
