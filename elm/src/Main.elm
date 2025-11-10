module Main exposing (main)

import Api
import Browser
import Html exposing (..)
import Html.Attributes exposing (..)
import Html.Events exposing (..)
import Json.Decode as Decode
import Task
import Time
import Types exposing (..)
import WebSocket


main : Program () Model Msg
main =
    Browser.element
        { init = init
        , update = update
        , subscriptions = subscriptions
        , view = view
        }


init : () -> ( Model, Cmd Msg )
init _ =
    ( { page = Dashboard
      , bookings = []
      , students = []
      , alerts = []
      , bookingsLoading = True
      , studentsLoading = True
      , alertsLoading = False
      , formSubmitting = False
      , error = Nothing
      , successMessage = Nothing
      , successMessageTime = Nothing
      , bookingFormErrors = []
      , studentFormErrors = []
      , newBookingForm = emptyBookingForm
      , newStudentForm = emptyStudentForm
      , websocketStatus = Connecting
      , rescheduleModal = Nothing
      }
    , Cmd.batch
        [ Api.getBookings GotBookings
        , Api.getStudents GotStudents
        ]
    )


emptyBookingForm : BookingForm
emptyBookingForm =
    { studentId = ""
    , aircraftType = ""
    , scheduledDate = ""
    , endTime = ""
    , locationName = ""
    , locationLat = ""
    , locationLon = ""
    }


emptyStudentForm : StudentForm
emptyStudentForm =
    { name = ""
    , email = ""
    , phone = ""
    , trainingLevel = ""
    }


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        ChangePage page ->
            ( { model | page = page }, Cmd.none )

        GotBookings result ->
            case result of
                Ok bookings ->
                    ( { model | bookings = bookings, formSubmitting = False }, Cmd.none )

                Err err ->
                    ( { model | error = Just err, formSubmitting = False }, Cmd.none )

        GotStudents result ->
            case result of
                Ok students ->
                    ( { model | students = students, formSubmitting = False }, Cmd.none )

                Err err ->
                    ( { model | error = Just err, formSubmitting = False }, Cmd.none )

        CreateBooking ->
            let
                errors = validateBookingForm model.newBookingForm
            in
            if List.isEmpty errors then
                ( { model
                    | formSubmitting = True
                    , bookingFormErrors = []
                    , successMessage = Nothing
                  }
                , Api.createBooking model.newBookingForm BookingCreated
                )
            else
                ( { model | bookingFormErrors = errors }, Cmd.none )

        BookingCreated result ->
            case result of
                Ok booking ->
                    ( { model
                        | bookings = booking :: model.bookings
                        , newBookingForm = emptyBookingForm
                        , formSubmitting = False
                        , bookingFormErrors = []
                      }
                    , Task.perform (SetSuccessMessage "Booking created successfully") Time.now
                    )

                Err err ->
                    ( { model
                        | error = Just err
                        , formSubmitting = False
                        , bookingFormErrors = [ { field = "general", message = err } ]
                      }
                    , Cmd.none
                    )

        CreateStudent ->
            let
                errors = validateStudentForm model.newStudentForm
            in
            if List.isEmpty errors then
                ( { model
                    | formSubmitting = True
                    , studentFormErrors = []
                    , successMessage = Nothing
                  }
                , Api.createStudent model.newStudentForm StudentCreated
                )
            else
                ( { model | studentFormErrors = errors }, Cmd.none )

        StudentCreated result ->
            case result of
                Ok student ->
                    ( { model
                        | students = student :: model.students
                        , newStudentForm = emptyStudentForm
                        , formSubmitting = False
                        , studentFormErrors = []
                      }
                    , Task.perform (SetSuccessMessage "Student created successfully") Time.now
                    )

                Err err ->
                    ( { model
                        | error = Just err
                        , formSubmitting = False
                        , studentFormErrors = [ { field = "general", message = err } ]
                      }
                    , Cmd.none
                    )

        UpdateBookingForm field value ->
            let
                form =
                    model.newBookingForm

                newForm =
                    case field of
                        StudentIdField ->
                            { form | studentId = value }

                        AircraftTypeField ->
                            { form | aircraftType = value }

                        ScheduledDateField ->
                            { form | scheduledDate = value }

                        EndTimeField ->
                            { form | endTime = value }

                        LocationNameField ->
                            { form | locationName = value }

                        LocationLatField ->
                            { form | locationLat = value }

                        LocationLonField ->
                            { form | locationLon = value }
            in
            ( { model | newBookingForm = newForm }, Cmd.none )

        UpdateStudentForm field value ->
            let
                form =
                    model.newStudentForm

                newForm =
                    case field of
                        NameField ->
                            { form | name = value }

                        EmailField ->
                            { form | email = value }

                        PhoneField ->
                            { form | phone = value }

                        TrainingLevelField ->
                            { form | trainingLevel = value }
            in
            ( { model | newStudentForm = newForm }, Cmd.none )

        ValidateBookingField field ->
            let
                fieldErrors = validateBookingFormField field model.newBookingForm

                -- Remove existing errors for this field and related fields
                fieldsToRemove = case field of
                    ScheduledDateField -> ["start-time", "end-time"]
                    EndTimeField -> ["end-time"]
                    _ -> [getFieldName field]

                otherErrors = List.filter
                    (\err -> not (List.member err.field fieldsToRemove))
                    model.bookingFormErrors

                newErrors = otherErrors ++ fieldErrors
            in
            ( { model | bookingFormErrors = newErrors }, Cmd.none )

        ValidateStudentField field ->
            let
                fieldErrors = validateStudentFormField field model.newStudentForm
                fieldName = getStudentFieldName field

                -- Remove existing errors for this field
                otherErrors = List.filter (\err -> err.field /= fieldName) model.studentFormErrors
                newErrors = otherErrors ++ fieldErrors
            in
            ( { model | studentFormErrors = newErrors }, Cmd.none )

        WebSocketMessageReceived message ->
            case Decode.decodeString Api.alertDecoder message of
                Ok alert ->
                    ( { model | alerts = alert :: model.alerts }, Cmd.none )

                Err _ ->
                    ( model, Cmd.none )

        WebSocketConnected ->
            ( { model | websocketStatus = Connected }, Cmd.none )

        WebSocketDisconnected ->
            ( { model | websocketStatus = Disconnected }, Cmd.none )

        DismissAlert alertId ->
            ( { model | alerts = List.filter (\a -> a.id /= alertId) model.alerts }
            , Cmd.none
            )

        ClearSuccessMessage ->
            ( { model | successMessage = Nothing, successMessageTime = Nothing }, Cmd.none )

        SetSuccessMessage message time ->
            ( { model | successMessage = Just message, successMessageTime = Just time }, Cmd.none )

        Tick currentTime ->
            case model.successMessageTime of
                Just messageTime ->
                    let
                        elapsed =
                            Time.posixToMillis currentTime - Time.posixToMillis messageTime
                    in
                    if elapsed > 5000 then
                        ( { model | successMessage = Nothing, successMessageTime = Nothing }, Cmd.none )
                    else
                        ( model, Cmd.none )

                Nothing ->
                    ( model, Cmd.none )

        OpenRescheduleModal booking ->
            ( { model
                | rescheduleModal =
                    Just
                        { booking = booking
                        , options = []
                        , loading = True
                        , selectedOption = Nothing
                        , showConfirmation = False
                        }
              }
            , Api.getRescheduleSuggestions booking.id GotRescheduleOptions
            )

        CloseRescheduleModal ->
            ( { model | rescheduleModal = Nothing }, Cmd.none )

        GotRescheduleOptions result ->
            case result of
                Ok options ->
                    case model.rescheduleModal of
                        Just modal ->
                            ( { model
                                | rescheduleModal =
                                    Just { modal | options = options, loading = False }
                              }
                            , Cmd.none
                            )

                        Nothing ->
                            ( model, Cmd.none )

                Err err ->
                    ( { model
                        | error = Just ("Failed to get reschedule options: " ++ err)
                        , rescheduleModal = Nothing
                      }
                    , Cmd.none
                    )

        SelectRescheduleOption option ->
            case model.rescheduleModal of
                Just modal ->
                    ( { model
                        | rescheduleModal =
                            Just { modal | selectedOption = Just option, showConfirmation = True }
                      }
                    , Cmd.none
                    )

                Nothing ->
                    ( model, Cmd.none )

        ShowRescheduleConfirmation ->
            case model.rescheduleModal of
                Just modal ->
                    ( { model
                        | rescheduleModal = Just { modal | showConfirmation = True }
                      }
                    , Cmd.none
                    )

                Nothing ->
                    ( model, Cmd.none )

        CancelRescheduleConfirmation ->
            case model.rescheduleModal of
                Just modal ->
                    ( { model
                        | rescheduleModal =
                            Just { modal | selectedOption = Nothing, showConfirmation = False }
                      }
                    , Cmd.none
                    )

                Nothing ->
                    ( model, Cmd.none )

        ConfirmReschedule ->
            case model.rescheduleModal of
                Just modal ->
                    case modal.selectedOption of
                        Just option ->
                            ( { model
                                | rescheduleModal = Just { modal | loading = True }
                              }
                            , Api.rescheduleBooking modal.booking.id option.dateTime RescheduleCompleted
                            )

                        Nothing ->
                            ( model, Cmd.none )

                Nothing ->
                    ( model, Cmd.none )

        RescheduleCompleted result ->
            case result of
                Ok updatedBooking ->
                    ( { model
                        | bookings =
                            List.map
                                (\b ->
                                    if b.id == updatedBooking.id then
                                        updatedBooking

                                    else
                                        b
                                )
                                model.bookings
                        , rescheduleModal = Nothing
                      }
                    , Task.perform (SetSuccessMessage "Booking rescheduled successfully") Time.now
                    )

                Err err ->
                    ( { model
                        | error = Just ("Failed to reschedule booking: " ++ err)
                        , rescheduleModal = Nothing
                      }
                    , Cmd.none
                    )


subscriptions : Model -> Sub Msg
subscriptions _ =
    Sub.batch
        [ WebSocket.websocketIn WebSocketMessageReceived
        , WebSocket.websocketConnected (\_ -> WebSocketConnected)
        , WebSocket.websocketDisconnected (\_ -> WebSocketDisconnected)
        , Time.every 10000 Tick
        ]


view : Model -> Html Msg
view model =
    div [ class "app" ]
        [ viewHeader model
        , viewNavigation model
        , viewAlerts model
        , viewContent model
        , case model.rescheduleModal of
            Just modal ->
                viewRescheduleModal modal

            Nothing ->
                text ""
        ]


viewHeader : Model -> Html Msg
viewHeader model =
    header [ class "header" ]
        [ h1 [] [ text "✈️ Flight Schedule Pro" ]
        , div [ class "status" ]
            [ viewWebSocketStatus model.websocketStatus ]
        ]


viewWebSocketStatus : WebSocketStatus -> Html Msg
viewWebSocketStatus status =
    case status of
        Connecting ->
            span [ class "status-badge connecting", attribute "data-testid" "ws-status" ] [ text "Connecting..." ]

        Connected ->
            span [ class "status-badge connected", attribute "data-testid" "ws-status" ] [ text "● Live" ]

        Disconnected ->
            span [ class "status-badge disconnected", attribute "data-testid" "ws-status" ] [ text "○ Disconnected" ]


viewNavigation : Model -> Html Msg
viewNavigation model =
    nav [ class "navigation" ]
        [ button
            [ class (navClass model Dashboard)
            , onClick (ChangePage Dashboard)
            , attribute "data-testid" "nav-dashboard"
            ]
            [ text "Dashboard" ]
        , button
            [ class (navClass model Bookings)
            , onClick (ChangePage Bookings)
            , attribute "data-testid" "nav-bookings"
            ]
            [ text "Bookings" ]
        , button
            [ class (navClass model Students)
            , onClick (ChangePage Students)
            , attribute "data-testid" "nav-students"
            ]
            [ text "Students" ]
        , button
            [ class (navClass model Alerts)
            , onClick (ChangePage Alerts)
            , attribute "data-testid" "nav-alerts"
            ]
            [ text "Alerts" ]
        ]


navClass : Model -> Page -> String
navClass model page =
    if model.page == page then
        "nav-button active"
    else
        "nav-button"


viewAlerts : Model -> Html Msg
viewAlerts model =
    if List.isEmpty model.alerts then
        text ""

    else
        div [ class "alerts-banner" ]
            (List.map viewAlert model.alerts)


viewAlert : Alert -> Html Msg
viewAlert alert =
    let
        severityClass =
            case alert.severity of
                Severe ->
                    "severe"

                High ->
                    "high"

                Moderate ->
                    "moderate"

                Low ->
                    "low"

                Clear ->
                    "clear"

        severityIcon =
            case alert.severity of
                Severe ->
                    "⛈️"

                High ->
                    "🌧️"

                Moderate ->
                    "⚡"

                Low ->
                    "🌤️"

                Clear ->
                    "☀️"

        locationText =
            if String.isEmpty alert.location then
                ""

            else
                " (" ++ alert.location ++ ")"

        timestampText =
            if String.isEmpty alert.timestamp then
                ""

            else
                " - " ++ formatTimestamp alert.timestamp
    in
    div
        [ class ("alert alert-" ++ severityClass)
        , attribute "data-testid" "weather-alert"
        ]
        [ div [ class "alert-content" ]
            [ span [ class "alert-icon" ] [ text (severityIcon ++ " ") ]
            , span [ class "alert-message" ]
                [ text (alert.message ++ locationText ++ timestampText) ]
            ]
        , button
            [ class "alert-dismiss"
            , attribute "data-testid" "dismiss-alert-btn"
            , onClick (DismissAlert alert.id)
            ]
            [ text "×" ]
        ]


formatTimestamp : String -> String
formatTimestamp timestamp =
    -- Simple formatting - just take the date/time part
    -- In production, you'd use a proper time library
    String.left 19 (String.replace "T" " " timestamp)


viewLoadingSpinner : String -> Html Msg
viewLoadingSpinner message =
    div [ class "loading-spinner", attribute "data-testid" "loading-spinner" ]
        [ div [ class "spinner" ] []
        , text message
        ]


viewContent : Model -> Html Msg
viewContent model =
    div [ class "content" ]
        [ case model.error of
            Just err ->
                div [ class "error", attribute "data-testid" "error-message" ] [ text ("Error: " ++ err) ]

            Nothing ->
                text ""
        , case model.successMessage of
            Just msg ->
                div [ class "success", attribute "data-testid" "success-message" ]
                    [ text msg
                    , button [ class "success-dismiss", onClick ClearSuccessMessage ] [ text "×" ]
                    ]

            Nothing ->
                text ""
        , case model.page of
            Dashboard ->
                viewDashboard model

            Bookings ->
                viewBookings model

            Students ->
                viewStudents model

            Alerts ->
                viewAlertsPage model
        ]


viewDashboard : Model -> Html Msg
viewDashboard model =
    div [ class "dashboard" ]
        [ h2 [] [ text "Dashboard" ]
        , div [ class "dashboard-stats" ]
            [ div [ class "stat-card", attribute "data-testid" "stat-bookings" ]
                [ h3 [] [ text (String.fromInt (List.length model.bookings)) ]
                , p [] [ text "Total Bookings" ]
                ]
            , div [ class "stat-card", attribute "data-testid" "stat-students" ]
                [ h3 [] [ text (String.fromInt (List.length model.students)) ]
                , p [] [ text "Students" ]
                ]
            , div [ class "stat-card", attribute "data-testid" "stat-alerts" ]
                [ h3 [] [ text (String.fromInt (List.length model.alerts)) ]
                , p [] [ text "Active Alerts" ]
                ]
            ]
        , h3 [] [ text "Recent Bookings" ]
        , viewBookingsList (List.take 5 model.bookings)
        ]


viewBookings : Model -> Html Msg
viewBookings model =
    div [ class "bookings-page" ]
        [ h2 [] [ text "Bookings" ]
        , button
            [ class "button button-primary"
            , onClick (ChangePage Bookings)  -- This will be replaced with a proper show form action
            , attribute "data-testid" "create-booking-btn"
            ]
            [ text "Create New Booking" ]
        , div [ class "form-card" ]
            [ h3 [] [ text "Create New Booking" ]
            , viewBookingForm model
            ]
        , h3 [] [ text "All Bookings" ]
        , viewBookingsList model.bookings
        ]


viewBookingForm : Model -> Html Msg
viewBookingForm model =
    div [ class "form" ]
        [ viewFormErrors model.bookingFormErrors
        , if model.formSubmitting then
            viewLoadingSpinner "Creating booking..."
          else
            text ""
        , div [ class "form-group" ]
            [ label [] [ text "Aircraft Type *" ]
            , select
                [ onInput (UpdateBookingForm AircraftTypeField)
                , onBlur (ValidateBookingField AircraftTypeField)
                , value model.newBookingForm.aircraftType
                , attribute "data-testid" "booking-aircraft"
                ]
                [ option [ value "" ] [ text "Select aircraft type" ]
                , option [ value "Cessna 172" ] [ text "Cessna 172" ]
                , option [ value "Piper Cherokee" ] [ text "Piper Cherokee" ]
                , option [ value "Diamond DA40" ] [ text "Diamond DA40" ]
                ]
            , viewFieldError "aircraft-type" model.bookingFormErrors
            ]
        , div [ class "form-group" ]
            [ label [] [ text "Student *" ]
            , select
                [ onInput (UpdateBookingForm StudentIdField)
                , onBlur (ValidateBookingField StudentIdField)
                , value model.newBookingForm.studentId
                ]
                (option [ value "" ] [ text "Select a student" ]
                    :: List.map
                        (\s -> option [ value s.id ] [ text s.name ])
                        model.students
                )
            , viewFieldError "student" model.bookingFormErrors
            ]
        , div [ class "form-group" ]
            [ label [] [ text "Scheduled Date (YYYY-MM-DDTHH:MM:SSZ) *" ]
            , input
                [ type_ "text"
                , placeholder "2024-01-15T14:00:00Z"
                , value model.newBookingForm.scheduledDate
                , onInput (UpdateBookingForm ScheduledDateField)
                , onBlur (ValidateBookingField ScheduledDateField)
                , attribute "data-testid" "booking-start-time"
                ]
                []
            , viewFieldError "start-time" model.bookingFormErrors
            ]
        , div [ class "form-group" ]
            [ label [] [ text "End Time (YYYY-MM-DDTHH:MM:SSZ) *" ]
            , input
                [ type_ "text"
                , placeholder "2024-01-15T16:00:00Z"
                , value model.newBookingForm.endTime
                , onInput (UpdateBookingForm EndTimeField)
                , onBlur (ValidateBookingField EndTimeField)
                , attribute "data-testid" "booking-end-time"
                ]
                []
            , viewFieldError "end-time" model.bookingFormErrors
            ]
        , div [ class "form-group" ]
            [ label [] [ text "Location Name *" ]
            , input
                [ type_ "text"
                , placeholder "KTOA"
                , value model.newBookingForm.locationName
                , onInput (UpdateBookingForm LocationNameField)
                , onBlur (ValidateBookingField LocationNameField)
                , attribute "data-testid" "booking-location"
                ]
                []
            , viewFieldError "location" model.bookingFormErrors
            ]
        , div [ class "form-row" ]
            [ div [ class "form-group" ]
                [ label [] [ text "Latitude" ]
                , input
                    [ type_ "text"
                    , placeholder "33.8113"
                    , value model.newBookingForm.locationLat
                    , onInput (UpdateBookingForm LocationLatField)
                    , onBlur (ValidateBookingField LocationLatField)
                    ]
                    []
                , viewFieldError "location-lat" model.bookingFormErrors
                ]
            , div [ class "form-group" ]
                [ label [] [ text "Longitude" ]
                , input
                    [ type_ "text"
                    , placeholder "-118.1515"
                    , value model.newBookingForm.locationLon
                    , onInput (UpdateBookingForm LocationLonField)
                    , onBlur (ValidateBookingField LocationLonField)
                    ]
                    []
                , viewFieldError "location-lon" model.bookingFormErrors
                ]
            ]
        , button
            [ class "button button-primary"
            , onClick CreateBooking
            , disabled (model.formSubmitting || not (List.isEmpty model.bookingFormErrors))
            , attribute "data-testid" "submit-booking-btn"
            ]
            [ text
                (if model.formSubmitting then
                    "Creating..."

                 else
                    "Create Booking"
                )
            ]
        ]


viewBookingsList : List Booking -> Html Msg
viewBookingsList bookings =
    if List.isEmpty bookings then
        p [ attribute "data-testid" "empty-bookings" ] [ text "No bookings found." ]

    else
        div [ class "bookings-list", attribute "data-testid" "booking-list" ]
            (List.map viewBookingCard bookings)


viewBookingCard : Booking -> Html Msg
viewBookingCard booking =
    div [ class ("booking-card status-" ++ String.toLower booking.status), attribute "data-testid" "booking-item" ]
        [ div [ class "booking-header" ]
            [ h4 [] [ text booking.departureLocation.name ]
            , span [ class "status-badge" ] [ text booking.status ]
            ]
        , p [] [ text ("Aircraft: " ++ booking.aircraftType) ]
        , p [] [ text ("Date: " ++ formatDateWithTimezone booking.scheduledDate) ]
        , p [] [ text ("Student ID: " ++ booking.studentId) ]
        , button
            [ class "button button-secondary"
            , attribute "data-testid" "reschedule-btn"
            , onClick (OpenRescheduleModal booking)
            ]
            [ text "Reschedule" ]
        ]


formatDateWithTimezone : String -> String
formatDateWithTimezone dateStr =
    dateStr ++ " UTC"


viewStudents : Model -> Html Msg
viewStudents model =
    div [ class "students-page" ]
        [ h2 [] [ text "Students" ]
        , button
            [ class "button button-primary"
            , onClick (ChangePage Students)  -- This will be replaced with a proper show form action
            , attribute "data-testid" "create-student-btn"
            ]
            [ text "Add New Student" ]
        , div [ class "form-card" ]
            [ h3 [] [ text "Add New Student" ]
            , viewStudentForm model
            ]
        , h3 [] [ text "All Students" ]
        , viewStudentsList model.students
        ]


viewStudentForm : Model -> Html Msg
viewStudentForm model =
    div [ class "form" ]
        [ viewFormErrors model.studentFormErrors
        , if model.formSubmitting then
            viewLoadingSpinner "Creating student..."
          else
            text ""
         , div [ class "form-group" ]
             [ label [] [ text "Name *" ]
             , input
                 [ type_ "text"
                 , placeholder "John Doe"
                 , value model.newStudentForm.name
                 , onInput (UpdateStudentForm NameField)
                 , onBlur (ValidateStudentField NameField)
                 , attribute "data-testid" "student-name"
                 ]
                 []
             , viewFieldError "name" model.studentFormErrors
             ]
         , div [ class "form-group" ]
             [ label [] [ text "Email *" ]
             , input
                 [ type_ "email"
                 , placeholder "john@example.com"
                 , value model.newStudentForm.email
                 , onInput (UpdateStudentForm EmailField)
                 , onBlur (ValidateStudentField EmailField)
                 , attribute "data-testid" "student-email"
                 ]
                 []
             , viewFieldError "email" model.studentFormErrors
             ]
         , div [ class "form-group" ]
             [ label [] [ text "Phone *" ]
             , input
                 [ type_ "tel"
                 , placeholder "(555) 123-4567"
                 , value model.newStudentForm.phone
                 , onInput (UpdateStudentForm PhoneField)
                 , onBlur (ValidateStudentField PhoneField)
                 , attribute "data-testid" "student-phone"
                 ]
                 []
             , viewFieldError "phone" model.studentFormErrors
             ]
        , div [ class "form-group" ]
            [ label [] [ text "Training Level *" ]
            , select
                [ onInput (UpdateStudentForm TrainingLevelField)
                , onBlur (ValidateStudentField TrainingLevelField)
                , value model.newStudentForm.trainingLevel
                , attribute "data-testid" "student-training-level"
                ]
                [ option [ value "" ] [ text "Select training level" ]
                , option [ value "STUDENT_PILOT" ] [ text "Student Pilot" ]
                , option [ value "PRIVATE_PILOT" ] [ text "Private Pilot" ]
                , option [ value "INSTRUMENT_RATED" ] [ text "Instrument Rated" ]
                ]
            , viewFieldError "training-level" model.studentFormErrors
            ]
        , button
            [ class "button button-primary"
            , onClick CreateStudent
            , disabled (model.formSubmitting || not (List.isEmpty model.studentFormErrors))
            , attribute "data-testid" "submit-student-btn"
            ]
            [ text
                (if model.formSubmitting then
                    "Creating..."

                 else
                    "Add Student"
                )
            ]
        ]


viewStudentsList : List Student -> Html Msg
viewStudentsList students =
    if List.isEmpty students then
        p [ attribute "data-testid" "empty-students" ] [ text "No students found." ]

    else
        div [ class "students-list", attribute "data-testid" "student-list" ]
            (List.map viewStudentCard students)


viewStudentCard : Student -> Html Msg
viewStudentCard student =
    div [ class "student-card", attribute "data-testid" "student-item" ]
        [ h4 [] [ text student.name ]
        , p [] [ text ("Email: " ++ student.email) ]
        , p [] [ text ("Phone: " ++ student.phone) ]
        , span [ class "badge" ] [ text student.trainingLevel ]
        ]


viewAlertsPage : Model -> Html Msg
viewAlertsPage model =
    div [ class "alerts-page" ]
        [ h2 [] [ text "Weather Alerts" ]
        , if List.isEmpty model.alerts then
            p [] [ text "No active alerts." ]

          else
            div [ class "alerts-list" ]
                (List.map viewAlertCard model.alerts)
        ]


viewAlertCard : Alert -> Html Msg
viewAlertCard alert =
    let
        severityIcon =
            case alert.severity of
                Severe ->
                    "⛈️"

                High ->
                    "🌧️"

                Moderate ->
                    "⚡"

                Low ->
                    "🌤️"

                Clear ->
                    "☀️"

        severityText =
            case alert.severity of
                Severe ->
                    "SEVERE"

                High ->
                    "HIGH"

                Moderate ->
                    "MODERATE"

                Low ->
                    "LOW"

                Clear ->
                    "CLEAR"
    in
    div [ class "alert-card" ]
        [ div [ class "alert-card-header" ]
            [ h4 [] [ text (severityIcon ++ " " ++ alert.alertType) ]
            , span [ class ("severity-badge severity-" ++ String.toLower severityText) ]
                [ text severityText ]
            ]
        , p [ class "alert-card-message" ] [ text alert.message ]
        , if not (String.isEmpty alert.location) then
            p [ class "alert-card-detail" ] [ text ("Location: " ++ alert.location) ]

          else
            text ""
        , if not (String.isEmpty alert.timestamp) then
            p [ class "alert-card-detail" ] [ text ("Time: " ++ formatTimestamp alert.timestamp) ]

          else
            text ""
        , case alert.studentName of
            Just name ->
                p [ class "alert-card-detail" ] [ text ("Student: " ++ name) ]

            Nothing ->
                text ""
        , case alert.originalDate of
            Just date ->
                p [ class "alert-card-detail" ] [ text ("Original Date: " ++ formatDateWithTimezone date) ]

            Nothing ->
                text ""
        , button
            [ class "button button-secondary"
            , onClick (DismissAlert alert.id)
            ]
            [ text "Dismiss" ]
        ]


viewFormErrors : List FormError -> Html Msg
viewFormErrors errors =
    case List.filter (\e -> e.field == "general") errors of
        [] ->
            text ""

        generalErrors ->
            div [ class "form-errors" ]
                (List.map (\e -> div [ class "error-message" ] [ text e.message ]) generalErrors)


viewFieldError : String -> List FormError -> Html Msg
viewFieldError fieldName errors =
    case List.filter (\e -> e.field == fieldName) errors of
        [] ->
            text ""

        fieldErrors ->
            div [ class "field-error", attribute "data-testid" ("error-" ++ fieldName) ]
                [ text (String.join ", " (List.map .message fieldErrors)) ]


validateStudentForm : StudentForm -> List FormError
validateStudentForm form =
    let
        nameErrors =
            if String.isEmpty (String.trim form.name) then
                [ { field = "name", message = "Name is required" } ]
            else if String.length (String.trim form.name) < 2 then
                [ { field = "name", message = "Name must be at least 2 characters" } ]
            else
                []

        emailErrors =
            if String.isEmpty (String.trim form.email) then
                [ { field = "email", message = "Email is required" } ]
            else if not (String.contains "@" form.email) then
                [ { field = "email", message = "Please enter a valid email address" } ]
            else
                []

        phoneErrors =
            if String.isEmpty (String.trim form.phone) then
                [ { field = "phone", message = "Phone is required" } ]
            else
                []

        trainingLevelErrors =
            if String.isEmpty form.trainingLevel then
                [ { field = "training-level", message = "Training level is required" } ]
            else
                []
    in
    nameErrors ++ emailErrors ++ phoneErrors ++ trainingLevelErrors


validateBookingForm : BookingForm -> List FormError
validateBookingForm form =
    let
        aircraftErrors =
            if String.isEmpty form.aircraftType then
                [ { field = "aircraft-type", message = "Aircraft type is required" } ]
            else
                []

        studentErrors =
            if String.isEmpty form.studentId then
                [ { field = "student", message = "Student selection is required" } ]
            else
                []

        dateErrors =
            if String.isEmpty (String.trim form.scheduledDate) then
                [ { field = "start-time", message = "Start time is required" } ]
            else
                []

        endTimeErrors =
            if String.isEmpty (String.trim form.endTime) then
                [ { field = "end-time", message = "End time is required" } ]
            else if not (String.isEmpty (String.trim form.scheduledDate)) && form.endTime < form.scheduledDate then
                [ { field = "end-time", message = "End time must be after start time" } ]
            else
                []

        locationErrors =
            if String.isEmpty (String.trim form.locationName) then
                [ { field = "location", message = "Location is required" } ]
            else
                []
    in
    aircraftErrors ++ studentErrors ++ dateErrors ++ endTimeErrors ++ locationErrors


-- Helper functions to get field names
getFieldName : BookingFormField -> String
getFieldName field =
    case field of
        AircraftTypeField -> "aircraft-type"
        StudentIdField -> "student"
        ScheduledDateField -> "start-time"
        EndTimeField -> "end-time"
        LocationNameField -> "location"
        LocationLatField -> "location-lat"
        LocationLonField -> "location-lon"


getStudentFieldName : StudentFormField -> String
getStudentFieldName field =
    case field of
        NameField -> "name"
        EmailField -> "email"
        PhoneField -> "phone"
        TrainingLevelField -> "training-level"


-- Individual field validation for real-time feedback
validateBookingFormField : BookingFormField -> BookingForm -> List FormError
validateBookingFormField field form =
    case field of
        AircraftTypeField ->
            if String.isEmpty form.aircraftType then
                [ { field = "aircraft-type", message = "Aircraft type is required" } ]
            else
                []

        StudentIdField ->
            if String.isEmpty form.studentId then
                [ { field = "student", message = "Student selection is required" } ]
            else
                []

        ScheduledDateField ->
            let
                dateErrors =
                    if String.isEmpty (String.trim form.scheduledDate) then
                        [ { field = "start-time", message = "Start time is required" } ]
                    else
                        []

                -- Also validate endTime when scheduledDate changes (cross-field)
                endTimeErrors =
                    if not (String.isEmpty (String.trim form.scheduledDate))
                        && not (String.isEmpty (String.trim form.endTime))
                        && form.endTime < form.scheduledDate then
                        [ { field = "end-time", message = "End time must be after start time" } ]
                    else
                        []
            in
            dateErrors ++ endTimeErrors

        EndTimeField ->
            if String.isEmpty (String.trim form.endTime) then
                [ { field = "end-time", message = "End time is required" } ]
            else if not (String.isEmpty (String.trim form.scheduledDate)) && form.endTime < form.scheduledDate then
                [ { field = "end-time", message = "End time must be after start time" } ]
            else
                []

        LocationNameField ->
            if String.isEmpty (String.trim form.locationName) then
                [ { field = "location", message = "Location is required" } ]
            else
                []

        LocationLatField ->
            if not (String.isEmpty (String.trim form.locationLat)) then
                case String.toFloat form.locationLat of
                    Nothing ->
                        [ { field = "location-lat", message = "Latitude must be a valid number" } ]
                    Just lat ->
                        if lat < -90 || lat > 90 then
                            [ { field = "location-lat", message = "Latitude must be between -90 and 90" } ]
                        else
                            []
            else
                []

        LocationLonField ->
            if not (String.isEmpty (String.trim form.locationLon)) then
                case String.toFloat form.locationLon of
                    Nothing ->
                        [ { field = "location-lon", message = "Longitude must be a valid number" } ]
                    Just lon ->
                        if lon < -180 || lon > 180 then
                            [ { field = "location-lon", message = "Longitude must be between -180 and 180" } ]
                        else
                            []
            else
                []


validateStudentFormField : StudentFormField -> StudentForm -> List FormError
validateStudentFormField field form =
    case field of
        NameField ->
            if String.isEmpty (String.trim form.name) then
                [ { field = "name", message = "Name is required" } ]
            else if String.length (String.trim form.name) < 2 then
                [ { field = "name", message = "Name must be at least 2 characters" } ]
            else if String.length (String.trim form.name) > 100 then
                [ { field = "name", message = "Name must be at most 100 characters" } ]
            else
                []

        EmailField ->
            if String.isEmpty (String.trim form.email) then
                [ { field = "email", message = "Email is required" } ]
            else if not (String.contains "@" form.email && String.contains "." form.email) then
                [ { field = "email", message = "Please enter a valid email address" } ]
            else
                []

        PhoneField ->
            if String.isEmpty (String.trim form.phone) then
                [ { field = "phone", message = "Phone is required" } ]
            else if String.length (String.trim form.phone) < 10 then
                [ { field = "phone", message = "Please enter a valid phone number (at least 10 digits)" } ]
            else
                []

        TrainingLevelField ->
            if String.isEmpty form.trainingLevel then
                [ { field = "training-level", message = "Training level is required" } ]
            else
                []


viewRescheduleModal : RescheduleModal -> Html Msg
viewRescheduleModal modal =
    div [ class "modal-overlay", onClick CloseRescheduleModal ]
        [ div [ class "modal-content" ]
            [ if modal.showConfirmation then
                viewConfirmationDialog modal

              else
                div []
                    [ div [ class "modal-header" ]
                        [ h3 [] [ text "Reschedule Flight" ]
                        , button
                            [ class "modal-close"
                            , onClick CloseRescheduleModal
                            ]
                            [ text "×" ]
                        ]
                    , div [ class "modal-body" ]
                        [ div [ class "booking-details" ]
                            [ p [] [ text ("Current time: " ++ formatDateWithTimezone modal.booking.scheduledDate) ]
                            , p [] [ text ("Location: " ++ modal.booking.departureLocation.name) ]
                            , p [] [ text ("Aircraft: " ++ modal.booking.aircraftType) ]
                            ]
                        , if modal.loading then
                            div [ attribute "data-testid" "reschedule-loading" ]
                                [ viewLoadingSpinner "Loading reschedule options..." ]

                          else if List.isEmpty modal.options then
                            div [ class "no-options" ]
                                [ text "No reschedule options available" ]

                          else
                            div [ class "reschedule-options" ]
                                (List.map (viewRescheduleOption modal.selectedOption) modal.options)
                        ]
                    ]
            ]
        ]


viewRescheduleOption : Maybe RescheduleOption -> RescheduleOption -> Html Msg
viewRescheduleOption selectedOption option =
    div
        [ class
            (if Just option == selectedOption then
                "reschedule-option selected"

             else
                "reschedule-option"
            )
        ]
        [ div [ class "option-time", attribute "data-testid" "option-time" ]
            [ text (formatDateWithTimezone option.dateTime) ]
        , div [ class "option-reason", attribute "data-testid" "option-reason" ]
            [ text option.reason ]
        , div [ class "option-badges" ]
            [ span
                [ class
                    (if option.instructorAvailable then
                        "badge badge-available"

                     else
                        "badge badge-unavailable"
                    )
                , attribute "data-testid" "availability-badge"
                ]
                [ text
                    (if option.instructorAvailable then
                        "Available"

                     else
                        "Unavailable"
                    )
                ]
            , span
                [ class
                    (if option.weatherScore >= 8.0 then
                        "badge badge-weather-good"

                     else if option.weatherScore >= 6.0 then
                        "badge badge-weather-marginal"

                     else
                        "badge badge-weather-poor"
                    )
                , attribute "data-testid" "weather-indicator"
                ]
                [ text
                    (if option.weatherScore >= 8.0 then
                        "Weather OK"

                     else if option.weatherScore >= 6.0 then
                        "Marginal"

                     else
                        "Not Suitable"
                    )
                ]
            ]
        , button
            [ class "button button-primary"
            , attribute "data-testid" "select-option-btn"
            , onClick (SelectRescheduleOption option)
            , disabled (not option.instructorAvailable)
            ]
            [ text "Select" ]
        ]


viewConfirmationDialog : RescheduleModal -> Html Msg
viewConfirmationDialog modal =
    case modal.selectedOption of
        Just option ->
            div [ attribute "data-testid" "confirm-reschedule-modal" ]
                [ div [ class "modal-header" ]
                    [ h3 [] [ text "Confirm Reschedule" ]
                    , button
                        [ class "modal-close"
                        , onClick CloseRescheduleModal
                        ]
                        [ text "×" ]
                    ]
                , div [ class "modal-body" ]
                    [ p [] [ text "Are you sure you want to reschedule this flight?" ]
                    , div [ class "change-summary" ]
                        [ p []
                            [ strong [] [ text "From: " ]
                            , text (formatDateWithTimezone modal.booking.scheduledDate)
                            ]
                        , p []
                            [ strong [] [ text "To: " ]
                            , text (formatDateWithTimezone option.dateTime)
                            ]
                        ]
                    , div [ class "modal-actions" ]
                        [ button
                            [ class "button button-secondary"
                            , attribute "data-testid" "cancel-reschedule-btn"
                            , onClick CancelRescheduleConfirmation
                            , disabled modal.loading
                            ]
                            [ text "Cancel" ]
                        , button
                            [ class "button button-primary"
                            , attribute "data-testid" "confirm-reschedule-btn"
                            , onClick ConfirmReschedule
                            , disabled modal.loading
                            ]
                            [ if modal.loading then
                                text "Rescheduling..."

                              else
                                text "Confirm"
                            ]
                        ]
                    ]
                ]

        Nothing ->
            text ""
