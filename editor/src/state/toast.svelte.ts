export type ToastType = 'success' | 'error' | 'info';

export interface ToastItem {
    id: number;
    message: string;
    type: ToastType;
    duration: number;
}

let nextId = 1;

function createToastState() {
    let toasts = $state<ToastItem[]>([]);

    function show(message: string, type: ToastType = 'info', duration: number = 3000) {
        const id = nextId++;
        toasts.push({ id, message, type, duration });
    }

    function dismiss(id: number) {
        const index = toasts.findIndex(t => t.id === id);
        if (index !== -1) {
            toasts.splice(index, 1);
        }
    }

    return {
        get toasts() { return toasts; },
        show,
        dismiss
    };
}

export const toastState = createToastState();
