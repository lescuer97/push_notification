// self.addEventListener('fetch', event => {
//   event.respondWith(
//     fetch(event.request)
//       .then(response => {
//         return response;
//       })
//       .catch(error => {
//         console.error('Error fetching:', error);
//       })
//   );
// });
//
self.addEventListener("activate", () => {
  console.log("activate");
  clients.claim();
});

self.addEventListener("install", () => {
  console.log("install");
  self.skipWaiting();
});


// Register event listener for the 'push' event.
self.addEventListener('push', function(event) {
    console.log({pushEvent: event});

    let message = event.data.json();
    console.log({message});
    /** @type {NotificationOptions} */
    let notifOptions = {body: message.data,
        data: "messge data",
        tag: "push_example",

    };
  // Keep the service worker alive until the notification is created.
  event.waitUntil(
    // Show a notification with title 'ServiceWorker Cookbook' and body 'Alea iacta est'.
    self.registration.showNotification(message.title, notifOptions)
  );
});
