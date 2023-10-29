// if ('serviceWorker' in navigator) {
  navigator.serviceWorker.register('/static/worker.js')
    .then(registration => {
      console.log('Service Worker registered with scope:', registration.scope);
    })
    .catch(error => {
      console.error('Error registering Service Worker:', error);
    });

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

    console.log({subsafter: subscription});
    


});
});


/** @type {HTMLInputElement} */
let general_notif_input = document.querySelector("input[name='general_notif']");

/** @type {HTMLInputElement} */
let back_stock_input = document.querySelector("input[name='back_stock']");


/** @type {HTMLButtonElement} */
let form = document.querySelector("form");

form.addEventListener("submit", async function(event) {
    event.preventDefault();
    const activeNotifs = [];

    form.querySelectorAll("input").forEach(function(input) {
        if (input.value==="on") {
            activeNotifs.push(input.name);

        }
    });
    console.log({activeNotifs});

    let result = await Notification.requestPermission();

    // subscriptionPush.action_condition = activeNotifs;

    console.log({subscriptionPush});

    let res = await fetch('/subscribe', {method: 'POST', body: JSON.stringify({subscription_push: subscriptionPush, action_condition: activeNotifs}), headers: {'Content-Type': 'application/json'}});
    console.log({res});

    // let res = await fetch("https://localhost:3000/"); let data = await res.json();

    // console.log({data});


    console.log("Submit button clicked!");
});

     
     

