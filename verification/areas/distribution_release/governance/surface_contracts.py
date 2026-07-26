"""Schema-v2 validation for install receipts and self CLI JSON surfaces."""

from __future__ import annotations

from typing import Any

from .common import (
    CHANNELS,
    TARGETS,
    fail,
    require_array,
    require_exact_keys,
    require_enum,
    require_nonempty_string,
    require_object,
    require_positive_int,
    require_schema_v2,
    require_sha256,
    version_channel,
)

RECEIPT_FIELDS = {
    "schema_version",
    "name",
    "version",
    "channel",
    "target",
    "install_dir",
    "binary_path",
    "sysroot_path",
    "sysroot_schema_version",
    "sysroot_sifr_version",
    "sysroot_target_triple",
    "sysroot_content_sha256",
    "artifact",
    "modify_path",
}
SELF_VERSION_FIELDS = {
    "schema_version",
    "current_executable",
    "current_version",
    "receipt_version",
    "install_dir",
    "binary_path",
    "sysroot_path",
    "sysroot_schema_version",
    "sysroot_sifr_version",
    "sysroot_target_triple",
    "channel",
    "target",
    "matches_receipt",
    "warnings",
}
UPDATE_PLAN_FIELDS = {
    "schema_version",
    "current_version",
    "target_version",
    "receipt_channel",
    "requested_channel",
    "resolved_channel",
    "install_dir",
    "binary_path",
    "sysroot_path",
    "installer_url",
    "action",
    "force",
    "would_run_installer",
    "warnings",
}


def validate_install_receipt(payload: Any) -> dict[str, Any]:
    receipt = require_object(payload, "$")
    require_exact_keys(receipt, required=RECEIPT_FIELDS, location="$")
    require_schema_v2(receipt)
    if receipt["name"] != "sifr":
        fail("$.name", "must be sifr")
    channel = validate_channel(receipt["channel"], "$.channel")
    if version_channel(receipt["version"], "$.version") != channel:
        fail("$.version", "does not match receipt channel")
    if receipt["target"] not in TARGETS:
        fail("$.target", "is not a supported target")
    for field in ("install_dir", "binary_path", "sysroot_path", "artifact"):
        require_nonempty_string(receipt[field], f"$.{field}")
    if (
        type(receipt["sysroot_schema_version"]) is not int
        or receipt["sysroot_schema_version"] != 1
    ):
        fail("$.sysroot_schema_version", "must be integer 1")
    if receipt["sysroot_sifr_version"] != receipt["version"]:
        fail("$.sysroot_sifr_version", "must match receipt version")
    if receipt["sysroot_target_triple"] != receipt["target"]:
        fail("$.sysroot_target_triple", "must match receipt target")
    require_sha256(receipt["sysroot_content_sha256"], "$.sysroot_content_sha256")
    if not isinstance(receipt["modify_path"], bool):
        fail("$.modify_path", "must be boolean")
    return receipt


def validate_self_version(payload: Any) -> dict[str, Any]:
    response = require_object(payload, "$")
    require_exact_keys(response, required=SELF_VERSION_FIELDS, location="$")
    require_schema_v2(response)
    channel = validate_channel(response["channel"], "$.channel")
    version_channel(response["current_version"], "$.current_version")
    if version_channel(response["receipt_version"], "$.receipt_version") != channel:
        fail("$.receipt_version", "does not match channel")
    if response["sysroot_sifr_version"] != response["receipt_version"]:
        fail("$.sysroot_sifr_version", "must match receipt version")
    for field in (
        "current_executable",
        "install_dir",
        "binary_path",
        "sysroot_path",
        "sysroot_target_triple",
        "target",
    ):
        require_nonempty_string(response[field], f"$.{field}")
    if (
        type(response["sysroot_schema_version"]) is not int
        or response["sysroot_schema_version"] != 1
    ):
        fail("$.sysroot_schema_version", "must be integer 1")
    if not isinstance(response["matches_receipt"], bool):
        fail("$.matches_receipt", "must be boolean")
    validate_warnings(response["warnings"])
    return response


def validate_self_update_plan(payload: Any) -> dict[str, Any]:
    plan = require_object(payload, "$")
    require_exact_keys(plan, required=UPDATE_PLAN_FIELDS, location="$")
    require_schema_v2(plan)
    current_channel = version_channel(plan["current_version"], "$.current_version")
    target_channel = version_channel(plan["target_version"], "$.target_version")
    receipt_channel = validate_channel(plan["receipt_channel"], "$.receipt_channel")
    resolved_channel = validate_channel(plan["resolved_channel"], "$.resolved_channel")
    if current_channel != receipt_channel:
        fail("$.current_version", "does not match receipt_channel")
    if target_channel != resolved_channel:
        fail("$.target_version", "does not match resolved_channel")
    requested = plan["requested_channel"]
    if requested is not None:
        requested = validate_channel(requested, "$.requested_channel")
        if requested != resolved_channel:
            fail("$.requested_channel", "must match resolved_channel")
    for field in ("install_dir", "binary_path", "sysroot_path", "installer_url"):
        require_nonempty_string(plan[field], f"$.{field}")
    action = require_enum(
        plan["action"],
        {"no_op", "update", "reinstall", "downgrade", "channel_switch"},
        "$.action",
    )
    if not isinstance(plan["force"], bool) or not isinstance(plan["would_run_installer"], bool):
        fail("$", "force and would_run_installer must be boolean")
    if plan["would_run_installer"] != (action != "no_op"):
        fail("$.would_run_installer", "does not agree with action")
    validate_warnings(plan["warnings"])
    return plan


def validate_channel(value: Any, location: str) -> str:
    return require_enum(value, set(CHANNELS), location)


def validate_warnings(value: Any) -> None:
    warnings = require_array(value, "$.warnings")
    for index, warning in enumerate(warnings):
        require_nonempty_string(warning, f"$.warnings[{index}]")
