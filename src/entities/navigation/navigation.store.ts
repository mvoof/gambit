import { makeAutoObservable } from 'mobx';

export type AppPage = 'home' | 'play' | 'archive' | 'settings';

export class NavigationStore {
  page: AppPage = 'home';
  /** Leaving the game would stop it, so the player is asked first. */
  isLeaveGuarded = false;
  pendingPage: AppPage | null = null;

  constructor() {
    makeAutoObservable(this);
  }

  setLeaveGuarded = (isGuarded: boolean) => {
    this.isLeaveGuarded = isGuarded;
  };

  /** A request from the player: it may wait for a confirmation. */
  request = (page: AppPage) => {
    if (this.page === 'play' && page !== 'play' && this.isLeaveGuarded) {
      this.pendingPage = page;
    } else {
      this.page = page;
    }
  };

  confirmLeave = () => {
    if (this.pendingPage) {
      this.page = this.pendingPage;
    }

    this.pendingPage = null;
  };

  cancelLeave = () => {
    this.pendingPage = null;
  };
}
