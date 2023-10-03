#!/bin/bash

if [ "$#" -ne 2 ]; then
    echo "❌ Usage: $0 <TASK_DEFINITION_NAME> <NEW_ARK_INDEXER_VERSION>"
    exit 1
fi

TASK_DEFINITION_NAME=$1
NEW_ARK_INDEXER_VERSION=$2

# Get the task definition excluding taskDefinitionArn and revision
TASK_DEFINITION=$(aws ecs describe-task-definition --task-definition $TASK_DEFINITION_NAME --query "taskDefinition | {family: family, containerDefinitions: containerDefinitions, volumes: volumes, placementConstraints: placementConstraints, networkMode: networkMode, executionRoleArn: executionRoleArn, memory: memory, cpu: cpu, requiresCompatibilities: requiresCompatibilities}")

CURRENT_ARK_INDEXER_VERSION=$(echo $TASK_DEFINITION | jq -r '.containerDefinitions[0].environment[] | select(.name=="ARK_INDEXER_VERSION") | .value')

if [ "$CURRENT_ARK_INDEXER_VERSION" = "$NEW_ARK_INDEXER_VERSION" ]; then
    echo "✅ The ARK_INDEXER_VERSION is already set to $NEW_ARK_INDEXER_VERSION. No action needed."
    exit 0
else
    echo "🔄 Current ARK_INDEXER_VERSION is $CURRENT_ARK_INDEXER_VERSION. Updating to $NEW_ARK_INDEXER_VERSION..."
   
    # Modify ARK_INDEXER_VERSION in the task definition
    NEW_TASK_DEFINITION=$(echo $TASK_DEFINITION | jq --arg NEW_ARK_INDEXER_VERSION "$NEW_ARK_INDEXER_VERSION" '
      .containerDefinitions[0].environment |= map(
        if .name == "ARK_INDEXER_VERSION" then
          .value = $NEW_ARK_INDEXER_VERSION
        else
          .
        end
      )
    ')

    echo $NEW_TASK_DEFINITION

    # NEW_TASK_DEF_REGISTER_RESULT=$(aws ecs register-task-definition --cli-input-json "$NEW_TASK_DEFINITION")

    # if [ $? -eq 0 ]; then
    #     echo "✨ Successfully registered new task definition with ARK_INDEXER_VERSION=$NEW_ARK_INDEXER_VERSION"
    # else
    #     echo "❌ Failed to register new task definition"
    #     exit 1
    # fi
fi
