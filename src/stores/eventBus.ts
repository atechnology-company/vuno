import { writable } from 'svelte/store';

type EventType = 'keydown' | 'keyup' | 'click';

interface EventBus {
  subscribe: (callback: (event: KeyboardEvent | MouseEvent) => void) => void;
  dispatch: (event: KeyboardEvent | MouseEvent) => void;
}

function createEventBus(): EventBus {
  const { subscribe, set } = writable<((event: KeyboardEvent | MouseEvent) => void)[]>([]);

  return {
    subscribe: (callback: (event: KeyboardEvent | MouseEvent) => void) => {
      subscribe(subscribers => {
        subscribers.push(callback);
        return subscribers;
      });

      return () => {
        subscribe(subscribers => {
          const index = subscribers.indexOf(callback);
          if (index > -1) {
            subscribers.splice(index, 1);
          }
          return subscribers;
        });
      };
    },
    dispatch: (event: KeyboardEvent | MouseEvent) => {
      subscribe(subscribers => {
        subscribers.forEach(callback => callback(event));
        return subscribers;
      });
    }
  };
}

export const eventBus = createEventBus(); 