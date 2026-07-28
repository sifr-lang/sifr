#!/usr/bin/env bash

set -euo pipefail

usage() {
  echo "usage: poll_site_release_run.sh --repository OWNER/REPO --workflow FILE --title TEXT --sha COMMIT --dispatched-at TIMESTAMP [--deadline-seconds N]" >&2
  exit 2
}

repository=""
workflow=""
title=""
sha=""
dispatched_at=""
deadline_seconds=1200
while [[ $# -gt 0 ]]; do
  case "$1" in
    --repository) repository="${2:-}"; shift 2 ;;
    --workflow) workflow="${2:-}"; shift 2 ;;
    --title) title="${2:-}"; shift 2 ;;
    --sha) sha="${2:-}"; shift 2 ;;
    --dispatched-at) dispatched_at="${2:-}"; shift 2 ;;
    --deadline-seconds) deadline_seconds="${2:-}"; shift 2 ;;
    *) usage ;;
  esac
done
[[ -n "${repository}" && -n "${workflow}" && -n "${title}" &&
  "${sha}" =~ ^[0-9a-f]{40}$ && -n "${dispatched_at}" &&
  "${deadline_seconds}" =~ ^[1-9][0-9]*$ ]] || usage

run_id=""
query_failures=0
poll_error=""
poll_deadline=$((SECONDS + deadline_seconds))
while (( SECONDS < poll_deadline )); do
  remaining_seconds=$((poll_deadline - SECONDS))
  (( remaining_seconds > 0 )) || break
  set +e
  runs="$(
    timeout --foreground "${remaining_seconds}s" gh api \
      --method GET \
      --paginate \
      --slurp \
      "repos/${repository}/actions/workflows/${workflow}/runs" \
      -f event=workflow_dispatch \
      -f created="${dispatched_at}..*" \
      -f per_page=100
  )"
  query_status=$?
  set -e
  [[ ${query_status} -ne 124 ]] || break
  if [[ ${query_status} -ne 0 ]]; then
    query_failures=$((query_failures + 1))
    if (( query_failures >= 3 )); then
      poll_error="could not query the correlated run after three attempts"
      break
    fi
  else
    query_failures=0
    matched="$(
      jq -r \
        --arg title "${title}" \
        --arg sha "${sha}" \
        --arg since "${dispatched_at}" \
        'first(
           .[].workflow_runs[]
           | select(.display_title == $title)
           | select(.head_sha == $sha)
           | select(.event == "workflow_dispatch")
           | select(.created_at >= $since)
           | [.id, .status, (.conclusion // "")]
           | @tsv
         ) // empty' <<<"${runs}"
    )"
    if [[ -n "${matched}" ]]; then
      IFS=$'\t' read -r run_id status conclusion <<<"${matched}"
      if [[ "${status}" == "completed" ]]; then
        if [[ "${conclusion}" == "success" ]]; then
          echo "Correlated site run ${run_id} completed successfully"
          exit 0
        fi
        echo "site-release-poll: correlated run ${run_id} concluded ${conclusion}" >&2
        exit 2
      fi
    fi
  fi
  remaining_seconds=$((poll_deadline - SECONDS))
  if (( remaining_seconds > 0 )); then
    sleep_seconds=10
    (( remaining_seconds >= sleep_seconds )) || sleep_seconds="${remaining_seconds}"
    sleep "${sleep_seconds}"
  fi
done
if [[ -n "${run_id}" ]]; then
  gh api --method POST "repos/${repository}/actions/runs/${run_id}/cancel" || true
fi
if [[ -n "${poll_error}" ]]; then
  echo "site-release-poll: ${poll_error}" >&2
  exit 2
fi
echo "site-release-poll: correlated deployment exceeded the deadline" >&2
exit 2
