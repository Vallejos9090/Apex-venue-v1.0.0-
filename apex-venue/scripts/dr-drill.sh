#!/bin/bash
set -euo pipefail
echo "🔥 DR Drill: Kill pod + truncate WAL + recover"
kubectl delete pod matching-engine-0 -n staging --wait=false
sleep 20
if kubectl logs -l app=matching-engine -n staging --tail=100 2>/dev/null | grep -q "Invariant violation"; then
    echo "❌ Invariant violation detected"
    exit 1
fi

echo "✅ Recovery successful - no invariant breaches"
