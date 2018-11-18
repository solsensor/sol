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
$ curl \
    https://solsensor.com/api/users/new \
    -XPOST \
    -H'Content-Type: application/json' \
    --data '{"email":"newuser@gmail.com","password":"mypassword"}'

{
    "status":"success",
    "message":"successfully created user"
}
```

Once your user is registered, you should see it at [https://solsensor.com/users].

### get a user token

A user token is needed to perform all api actions that require user
authentication (for example, adding a new sensor to your account). In order to
get a user token, you need to provide your email and password.

```
$ curl \
    https://solsensor.com/api/token \
    -XPOST \
    -u 'newuser@gmail.com:mypassword'

{
    "status":"success",
    "message":"got user token",
    "data":{
        "token":"user-DTYZYI5YSsH7hnIZrofhe2HhsaI4yZfgiPgED6pHSToiQI0wHWz9RqK8oPaZ3sMV"
    }
}
```

The token that you get back will be different from the one shown here. Make sure
to use the correct token in the following steps.

### add a sensor

To add a sensor to your account, provided your user token in the `Authorization`
header, and the hardware id of the sensor in the body.

```
$ curl \
    https://solsensor.com/api/add_sensor \
    -XPOST \
    -H'Authorization: bearer DTYZYI5YSsH7hnIZrofhe2HhsaI4yZfgiPgED6pHSToiQI0wHWz9RqK8oPaZ3sMV' \
    -H'Content-Type: application/json' \
    --data '{"hardware_id":1234567}'

{
    "status":"success",
    "message":"successfully added sensor"
}
```

Once you have added the sensor, log in at [https://solsensor.com/login], and
then navigate to your profile page at
[https://solsensor.com/user/newuser@gmail.com]. You should see the newly-added
sensor there.

### get a sensor token

Some actions (like adding a new reading) can only be performed by sensors. These
endpoints are authenticated via a sensor token. To get a sensor token, you need
to specify the hardware id of the sensor for which you want to get a token, and
the user token you provide must correspond to the sensor's owner.

```
$ curl \
    https://solsensor.com/api/sensor_token \
    -XPOST \
    -H'Authorization: bearer user-DTYZYI5YSsH7hnIZrofhe2HhsaI4yZfgiPgED6pHSToiQI0wHWz9RqK8oPaZ3sMV' \
    -H'Content-Type: application/json' \
    --data '{"hardware_id":1234567}'

{
    "message":"got sensor token",
    "status":"success",
    "data":{
        "token":"sensor-XY1cvYRLkrJFlIEQMyr03TWPeIzGsYIvriLySNJ4MI37SNpHWpBTVgy18ws7T9Ix"
    }
}
```

### add a reading

Once you have a sensor token, you can add a reading.

```
$ curl \
    https://solsensor.com/api/add_reading \
    -XPOST \
    -H'Authorization: bearer sensor-XY1cvYRLkrJFlIEQMyr03TWPeIzGsYIvriLySNJ4MI37SNpHWpBTVgy18ws7T9Ix' \
    -H'Content-Type: application/json' \
    --data '{"peak_power_mW":1.23,"peak_current_mA":1.23,"peak_voltage_V":1.23,"temp_celsius":15.2,"batt_V":1.23,"timestamp":1542513093}'

{
    "status":"success",
    "message":"successfully added reading"
}
```

You can also add multiple readings at once.

```
$ curl \
    https://solsensor.com/api/add_readings \
    -XPOST \
    -H'Authorization: bearer sensor-XY1cvYRLkrJFlIEQMyr03TWPeIzGsYIvriLySNJ4MI37SNpHWpBTVgy18ws7T9Ix' \
    -H'Content-Type: application/json' \
    --data '[{"peak_power_mW":1.23,"peak_current_mA":1.23,"peak_voltage_V":1.23,"temp_celsius":15.2,"batt_V":1.23,"timestamp":1542513093},{"peak_power_mW":1.23,"timestamp":1542513093,"peak_current_mA":1.23,"peak_voltage_V":1.23,"temp_celsius":15.2,"batt_V":1.23}]'

{
    "status":"success",
    "message":"successfully added readings"
}
```

Once the reading has been added, you should be able to see it in the web ui.
