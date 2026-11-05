import { describe, expect, it } from 'vitest';

import { NavigationStore } from './navigation.store';

describe('NavigationStore', () => {
  it('opens on the home page and goes where it is asked', () => {
    const navigation = new NavigationStore();

    expect(navigation.page).toBe('home');

    navigation.request('archive');

    expect(navigation.page).toBe('archive');
  });

  it('asks before leaving a running game', () => {
    const navigation = new NavigationStore();

    navigation.request('play');
    navigation.setLeaveGuarded(true);
    navigation.request('settings');

    expect(navigation.page).toBe('play');
    expect(navigation.pendingPage).toBe('settings');

    navigation.confirmLeave();

    expect(navigation.page).toBe('settings');
    expect(navigation.pendingPage).toBeNull();
  });

  it('stays in the game when the player changes their mind', () => {
    const navigation = new NavigationStore();

    navigation.request('play');
    navigation.setLeaveGuarded(true);
    navigation.request('home');
    navigation.cancelLeave();

    expect(navigation.page).toBe('play');
    expect(navigation.pendingPage).toBeNull();
  });

  it('never stops anyone from coming to the game', () => {
    const navigation = new NavigationStore();

    navigation.setLeaveGuarded(true);
    navigation.request('play');

    expect(navigation.page).toBe('play');
    expect(navigation.pendingPage).toBeNull();
  });
});
