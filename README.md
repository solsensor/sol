# sol

This repository contains the server that runs the solsensor.com website and API.
More detail will be included at a later time, but the following secions will
walk through basic interaction with the API.

## Walkthrough

### visiting the site

Visiting [https://solsensor.com/] takes you to the website's home page. From
here, you can create a new account and view details about users and sensors.

### register a new user

```
$ curl https://solsensor.com/api/users/new -XPOST -H'Content-Type: application/json' --data '{"email":"newuser@gmail.com","password":"mypassword"}'
{"status":"success"}
```

Once your user is registered, you should see it at [https://solsensor.com/users].

### get a user token

A user token is needed to perform all api actions that require user
authentication (for example, adding a new sensor to your account). In order to
get a user token, you need to provide your email and password.

```
$ curl https://solsensor.com/api/token -XPOST -H'Content-Type: application/json' --data '{"email":"newuser@gmail.com","password":"mypassword"}'
abcde12345
```

The token that you get back will be much longer than the one shown here. This
one is kept short for the sake of demonstration.

### add a sensor

To add a sensor to your account, provided your user token in the `Authorization`
header, and the hardware id of the sensor in the body.

```
$ curl https://solsensor.com/api/add_sensor -XPOST -H'Authorization: bearer abcde12345' -H'Content-Type: application/json' --data '{"hardware_id":1234567}'
success!
```

Once you have added the sensor, log in at [https://solsensor.com/login], and
then navigate to your profile page at
[https://solsensor.com/user/newuser@gmail.com]. You should see the newly-added
sensor there.

### get a sensor token

Some actions (like adding a new reading) can only be performed by sensors. These
endpoints are authenticated via a sensor token. To get a sensor token, you need
to specify the sensor for which you want to get a token, and the user token you
provide must correspond to the sensor's owner.

```
$ curl https://solsensor.com/api/sensor_token -XPOST -H'Authorization: bearer abcde12345' -H'Content-Type: application/json' --data '{"id":1,"owner_id":1,"hardware_id":1234567}'
wxyz6789
```

### add a reading

Once you have a sensor token, you can add a reading.

```
$ curl https://solsensor.com/api/add_reading -XPOST -H'Authorization: bearer wxyz6789' -H'Content-Type: application/json' --data '{"voltage":1.23}'
success!
```

Once the reading has been added, you should be able to see it in the web ui.
