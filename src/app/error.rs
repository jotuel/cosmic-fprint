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
