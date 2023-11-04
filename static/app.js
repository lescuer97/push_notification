import { changeDOMCheckboxValues, getCheckedInputs } from "./dom.js";

  navigator.serviceWorker.register('/static/worker.js')
    .then(registration => {
      console.log('Service Worker registered with scope:', registration.scope);
    })
    .catch(error => {
      console.error('Error registering Service Worker:', error);
    });


/** @type {PushSubscription} */
let subscriptionPush = null;

navigator.serviceWorker.ready.then(async function(registration) {
    // Use the PushManager to get the user's subscription to the push service.
  return registration.pushManager.getSubscription().then( async function(subscription) {

     // If a subscription was found, return it.
    if (subscription) {
      return subscription;
    }

    // get public key from server
    let res = await fetch('/pkey')

    let data = await res.arrayBuffer()

    return registration.pushManager.subscribe({
      userVisibleOnly: true,
      applicationServerKey: data,
    });

}).then( async function(subscription) {
    subscriptionPush = subscription;
    
    let subs = await set_subscription();
     
    for (let index = 0; index < subs.length; index++) {
        const element = subs[index];
        changeDOMCheckboxValues(element);
    }
    });
});



/** @type {HTMLFormElement} */
let form = document.querySelector("form");

form.addEventListener("submit", async function(event) {
    event.preventDefault();
    let result = await Notification.requestPermission();
     
    let checkedSubs = getCheckedInputs("notif-form");


    let res = await fetch('/subscribe', {method: 'POST', body: JSON.stringify({subscription_push: subscriptionPush, action_condition: checkedSubs}), headers: {'Content-Type': 'application/json'}});
});

let cancel = document.getElementById("unsubscribe");

cancel.addEventListener("click", async function(event) {
    event.preventDefault();

    let successful = await subscriptionPush.unsubscribe().catch(function(e) {
        console.log({e});
    });

    console.log({successful});

});


/** 
    * @async
    * Get all subscriptions
    * @returns { Promise<string[]>} */
async function set_subscription() {
    let res = await fetch(`/subscriptions?endpoint=${subscriptionPush.endpoint}`)
      
    if (res.ok) {
        let subs = await res.json()
        return subs;
    }
    
    return [];
}

