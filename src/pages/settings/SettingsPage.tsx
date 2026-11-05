import {
  Database,
  Cpu,
  Grid3x3,
  Monitor,
  Palette,
  Settings,
  Volume2,
} from 'lucide-react';
import { useState } from 'react';
import type { ReactNode } from 'react';
import { useTranslation } from 'react-i18next';

import { AnalysisSettings } from '@features/analysis-settings/AnalysisSettings';
import { UpdateSettings } from '@features/app-update/UpdateSettings';
import { BoardCustomise } from '@features/board-customise/BoardCustomise';
import { BoardPreview } from '@features/board-customise/BoardPreview';
import { DataSettings } from '@features/data-settings/DataSettings';
import { EngineSetup } from '@features/engine-setup/EngineSetup';
import { GeneralSettings } from '@features/general-settings/GeneralSettings';
import { SoundSettings } from '@features/general-settings/SoundSettings';
import { ThemeSettings } from '@features/general-settings/ThemeSettings';
import { PieceImport } from '@features/piece-import/PieceImport';
import { PieceSetPicker } from '@features/piece-import/PieceSetPicker';
import { PageHeader } from '@shared/ui/PageHeader/PageHeader';
import { Panel } from '@shared/ui/Panel/Panel';
import { SideNav } from '@shared/ui/SideNav/SideNav';

import styles from './SettingsPage.module.scss';

type SettingsSection =
  | 'general'
  | 'interface'
  | 'engine'
  | 'board'
  | 'sound'
  | 'data';

const SECTION_ICONS = {
  general: Monitor,
  interface: Palette,
  engine: Cpu,
  board: Grid3x3,
  sound: Volume2,
  data: Database,
};

const SECTIONS = Object.keys(SECTION_ICONS) as SettingsSection[];

const SectionBody = ({ section }: { section: SettingsSection }) => {
  const { t } = useTranslation();
  const bodies: Record<SettingsSection, ReactNode> = {
    general: (
      <>
        <Panel title={t('settings.general')}>
          <GeneralSettings />
        </Panel>
        <Panel title={t('settings.updates')}>
          <UpdateSettings />
        </Panel>
      </>
    ),
    interface: (
      <Panel title={t('settings.interface')}>
        <ThemeSettings />
      </Panel>
    ),
    engine: (
      <>
        <Panel title={t('settings.engine')}>
          <EngineSetup />
        </Panel>
        <Panel title={t('settings.analysis')}>
          <AnalysisSettings />
        </Panel>
      </>
    ),
    board: (
      <>
        <Panel title={t('settings.board')}>
          <div className={styles.boardSection}>
            <BoardCustomise />
            <BoardPreview />
          </div>
        </Panel>
        <Panel title={t('settings.pieces')}>
          <PieceSetPicker />
          <PieceImport />
        </Panel>
      </>
    ),
    sound: (
      <Panel title={t('settings.sound')}>
        <SoundSettings />
      </Panel>
    ),
    data: (
      <Panel title={t('settings.data')}>
        <DataSettings />
      </Panel>
    ),
  };

  return bodies[section];
};

export const SettingsPage = () => {
  const { t } = useTranslation();
  const [section, setSection] = useState<SettingsSection>('general');

  return (
    <main className={styles.page}>
      <PageHeader
        icon={Settings}
        title={t('settings.title')}
        subtitle={t('settings.subtitle')}
      />
      <div className={styles.body}>
        <aside className={styles.sidebar}>
          <SideNav
            label={t('settings.sections')}
            value={section}
            onChange={setSection}
            items={SECTIONS.map((value) => ({
              value,
              icon: SECTION_ICONS[value],
              label: t(`settings.${value}`),
            }))}
          />
        </aside>
        <div className={styles.content}>
          <SectionBody section={section} />
        </div>
      </div>
    </main>
  );
};
