"use client";

import React, {
  createContext,
  useContext,
  useState,
  type ReactNode,
} from "react";

export type TaskFormState = {
  from: string;
  to: string;
  numberOfTasks: string;
};

interface TaskFormContextType {
  state: TaskFormState;
  setState: React.Dispatch<React.SetStateAction<TaskFormState>>;
}

const TaskFormContext = createContext<TaskFormContextType | undefined>(
  undefined,
);

interface NetworkProviderProps {
  children: ReactNode;
}

export const TaskFormProvider: React.FC<NetworkProviderProps> = ({
  children,
}) => {
  const [state, setState] = useState<TaskFormState>({
    from: "0",
    to: "1000",
    numberOfTasks: "3",
  });

  return (
    <TaskFormContext.Provider value={{ state, setState }}>
      {children}
    </TaskFormContext.Provider>
  );
};

export const useTaskForm = (): TaskFormContextType => {
  const context = useContext(TaskFormContext);

  if (!context) {
    throw new Error("useNetwork must be used within a NetworkProvider");
  }

  return context;
};
