
/** if the given string has a dom input name change checked to true
    * @param {string} name 
    *
 */
export function changeDOMCheckboxValues(name) {
    
    /** @type {HTMLInputElement} */
    let domElement = document.querySelector(`input[name='${name}']`);
    
    if (domElement) {
        domElement.checked = true;
    }
}

/** check any on the inputs inside a form and extract checkbox values and names 
    * @param {string} formId
    * @returns {string[]}
    */
export function getCheckedInputs(formId) {

    /** @type {HTMLFormElement} */
    const form = document.querySelector(`form#${formId}`);

    /** @type {NodeListOf<HTMLInputElement>} */
    let inputs = form.querySelectorAll("input");
    

    /** @type {string[]} */
    const activeNotifs = [];

    for (let input of Array.from(inputs)) {
        if (input.checked) {
            activeNotifs.push([input.name, true]);
        } else {
            activeNotifs.push([input.name, false]);
        }
    }
    
    return activeNotifs;
}
