/**
 * Whether a keyboard event target is a form field that should receive its own key presses.
 * Should be used on stuff like global keyboard controls to ignore presses when a text input is selected.
 */
export function isEditableTarget(target: EventTarget | null) {
    return (
        target instanceof HTMLElement &&
        (target.isContentEditable || ["INPUT", "TEXTAREA", "SELECT"].includes(target.tagName))
    );
}
