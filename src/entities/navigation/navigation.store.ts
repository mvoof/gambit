import { makeAutoObservable } from 'mobx';

export type AppPage = 'play' | 'archive' | 'settings';

export class NavigationStore {
  page: AppPage = 'play';

  constructor() {
    makeAutoObservable(this);
  }

  open = (page: AppPage) => {
    this.page = page;
  };
}
