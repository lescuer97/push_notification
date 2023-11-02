# Push notification with app server and small front end. 

This is an experimental server on how to handle push notifications. It will just simply handle a subscription request and in the case a notification condition happens(just a simple http call in this case) send a push notification to the different browsers.

### Development 
 
#### Set up the DB

We need to set up the db, Vapid and TLS certs.
  
for now we need to have 3 cert files for the server to work. cert.pem, key.pem, vapid/private.key. 
for vapid/private.key if this file is not available it will create one. This file is needed to encrypt the data sent to the notification and what authentificates the application server to the push notification
servers.

for the tls files *cert.pem* and *key.pem*, this files will need to be imported to the working directory.

after this files are set up. We need to set up the Database:

```
# Install the sqlite Database(for ubuntu/Debian)

sudo apt-get install libsqlite3-dev

# create the DB 
bash db/setup-db.sh

# start the server 
cargo run

```
#### Register a  web push notification 

Visit the website: https://localhost:3000/static/index.html.

There is a form there where when you can select to which event to subscribe and then click the button to create a new subscription and send it to the server.

You might need to approve notifications the first time.


#### Triggering a http notification
Just visit the the url https://localhost:3000/send_push and add a query for the action that you want.  ex: https://localhost:3000/send_push?action=general_notif

Right now the only two triggering actions available are: general_notif and back_stock



