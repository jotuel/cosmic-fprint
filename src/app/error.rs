// SPDX-License-Identifier: MPL-2.0

use crate::fl;

#[derive(Debug, Clone, PartialEq)]
pub enum AppError {
    PermissionDenied,
    AlreadyInUse,
    Internal,
    NoEnrolledPrints,
    ClaimDevice,
    PrintsNotDeleted,
    Timeout,
    DeviceNotFound,
    ConnectDbus(String),
    Unknown(String),
}

impl AppError {
    pub fn localized_message(&self) -> String {
        match self {
            AppError::PermissionDenied => fl!("error-permission-denied"),
            AppError::AlreadyInUse => fl!("error-already-in-use"),
            AppError::Internal => fl!("error-internal"),
            AppError::NoEnrolledPrints => fl!("error-no-enrolled-prints"),
            AppError::ClaimDevice => fl!("error-claim-device"),
            AppError::PrintsNotDeleted => fl!("error-prints-not-deleted"),
            AppError::Timeout => fl!("error-timeout"),
            AppError::DeviceNotFound => fl!("error-device-not-found"),
            AppError::ConnectDbus(msg) => fl!("error-connect-dbus", err = msg),
            AppError::Unknown(msg) => msg.clone(),
        }
    }

    pub fn with_context(self, context: &str) -> Self {
        match self {
            AppError::Unknown(msg) => AppError::Unknown(format!("{}: {}", context, msg)),
            _ => self,
        }
    }
}

impl From<zbus::Error> for AppError {
    fn from(err: zbus::Error) -> Self {
        if let zbus::Error::MethodError(name, _, _) = &err {
            match name.as_str() {
                "net.reactivated.Fprint.Error.PermissionDenied" => AppError::PermissionDenied,
                "net.reactivated.Fprint.Error.AlreadyInUse" => AppError::AlreadyInUse,
                "net.reactivated.Fprint.Error.Internal" => AppError::Internal,
                "net.reactivated.Fprint.Error.NoEnrolledPrints" => AppError::NoEnrolledPrints,
                "net.reactivated.Fprint.Error.ClaimDevice" => AppError::ClaimDevice,
                "net.reactivated.Fprint.Error.PrintsNotDeleted" => AppError::PrintsNotDeleted,
                "net.reactivated.Fprint.Error.Timeout" => AppError::Timeout,
                "net.reactivated.Fprint.Error.DeviceNotFound" => AppError::DeviceNotFound,
                _ => AppError::Unknown(err.to_string()),
            }
        } else {
            AppError::Unknown(err.to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zbus::names::ErrorName;
    use zbus::message::Message;

    fn create_method_error(name: &str) -> zbus::Error {
        let msg = Message::method_call("/", "Ping")
            .unwrap()
            .destination("org.freedesktop.DBus")
            .unwrap()
            .build(&())
            .unwrap();

        let error_name = ErrorName::try_from(name).unwrap();
        // zbus::Error::MethodError(OwnedErrorName, Option<String>, Message)
        zbus::Error::MethodError(error_name.into(), None, msg)
    }

    #[test]
    fn test_zbus_error_conversion() {
        let test_cases = vec![
            ("net.reactivated.Fprint.Error.PermissionDenied", AppError::PermissionDenied),
            ("net.reactivated.Fprint.Error.AlreadyInUse", AppError::AlreadyInUse),
            ("net.reactivated.Fprint.Error.Internal", AppError::Internal),
            ("net.reactivated.Fprint.Error.NoEnrolledPrints", AppError::NoEnrolledPrints),
            ("net.reactivated.Fprint.Error.ClaimDevice", AppError::ClaimDevice),
            ("net.reactivated.Fprint.Error.PrintsNotDeleted", AppError::PrintsNotDeleted),
            ("net.reactivated.Fprint.Error.Timeout", AppError::Timeout),
            ("net.reactivated.Fprint.Error.DeviceNotFound", AppError::DeviceNotFound),
        ];

        for (error_str, expected) in test_cases {
            let zbus_err = create_method_error(error_str);
            let app_err = AppError::from(zbus_err);
            assert_eq!(app_err, expected, "Failed for error: {}", error_str);
        }
    }

    #[test]
    fn test_unknown_zbus_error() {
        let error_str = "net.reactivated.Fprint.Error.UnknownOne";
        let zbus_err = create_method_error(error_str);
        let app_err = AppError::from(zbus_err);

        if let AppError::Unknown(msg) = app_err {
            assert!(msg.contains(error_str));
        } else {
            panic!("Expected AppError::Unknown, got {:?}", app_err);
        }
    }

    #[test]
    fn test_non_method_error() {
        // Test a different zbus::Error variant
        let zbus_err = zbus::Error::from(std::io::Error::new(std::io::ErrorKind::Other, "test error"));
        let app_err = AppError::from(zbus_err);

        if let AppError::Unknown(msg) = app_err {
            assert!(msg.contains("test error"));
        } else {
            panic!("Expected AppError::Unknown, got {:?}", app_err);
        }
    }
}
